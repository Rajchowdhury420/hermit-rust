use axum::extract::ws::{Message, WebSocket};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

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
    let server_lock = server.lock().await;

    let ag_name = args[1].to_owned();

    match args[2].as_str() {
        "screenshot" => {
            // Set task
            if let Ok(_) = write_file(
                format!("agents/{}/task/name", ag_name.to_owned()),
                args[2..].join(" ").as_bytes()
            ) {
                info!("Task set successfully.");
            } else {
                error!("Task could not be set.");
                let _ = socket_lock.send(Message::Text("[task:error] Could not set a task.".to_owned())).await;
                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                return;
            }

            // Check the task result
            let mut cnt: u8 = 0;
            loop {
                info!("Getting task result...");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                if let Ok(task_result) = read_file(
                    format!("agents/{}/task/result", ag_name.to_owned())
                ) {
                    if task_result.len() > 0 {
                        info!("task result found.");
                        let outfile = format!(
                            "{}/agents/{}/screenshots/screenshot_1.png",
                            server_lock.config.app_dir.display(),
                            ag_name.to_owned(),
                        );
                        let _ = socket_lock.send(Message::Text(
                            format!("[task:screenshot:ok] {}", outfile))).await;
                        let _ = socket_lock.send(Message::Binary(task_result)).await;
                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

                        // Initialize the task result
                        empty_file(format!("agents/{}/task/result", ag_name.to_owned())).unwrap();
                        break;
                    } else {
                        warn!("task result is empty.");
                        cnt += 1;
                        if cnt > 5 {
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
        "shell" => {
            // Set task
            if let Ok(_) = write_file(format!("agents/{}/task/name", ag_name.to_owned()), args[2..].join(" ").as_bytes()) {
                info!("Task set successfully.");
            } else {
                error!("Task could not be set.");
                let _ = socket_lock.send(Message::Text("[task:error] Could not set a task.".to_owned())).await;
                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                return;
            }

            // Check the task result
            let mut cnt: u8 = 0;
            loop {
                info!("Getting task result...");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                if let Ok(task_result) = read_file(format!("agents/{}/task/result", ag_name.to_owned())) {
                    if task_result.len() > 0 {
                        info!("task result found.");
                        let _ = socket_lock.send(Message::Text("[task:shell:ok]".to_owned())).await;
                        let _ = socket_lock.send(Message::Binary(task_result)).await;
                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

                        // Initialize the task result
                        empty_file(format!("agents/{}/task/result", ag_name.to_owned())).unwrap();
                        break;
                    } else {
                        warn!("task result is empty.");
                        cnt += 1;
                        if cnt > 5 {
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
        _ => {
            let _ = socket_lock.send(Message::Text(format!("Unknown command: {message_text}"))).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
            return;
        }
    }
}