use axum::extract::ws::{Message, WebSocket};
use log::error;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use url::Url;

use crate::server::{
    jobs::{check_dupl_job, find_job, format_listeners, Job, JobMessage},
    server::Server,
};

pub async fn handle_listener(
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>,
) {
    let server_lock = server.lock().await;

    match args[1].as_str() {
        "add" => {
            let name = &args[2];
            let url = Url::parse(&args[3]).unwrap();

            let mut jobs = server_lock.jobs.lock().await;

            // Check if the url already exists.
            match check_dupl_job(&mut jobs, url.to_owned()) {
                Ok(_) => {},
                Err(e) => {
                    error!("{e}");
                    let _ = socket_lock.send(Message::Text(format!("Error: This URL already exists."))).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                },
            }

            let next_id = jobs.len() as u32;
            let rx_job = server_lock.tx_job.lock().await;

            let new_job = Job::new(
                next_id,
                name.to_owned(),
                url.scheme().to_owned(),
                url.host().unwrap().to_string(),
                url.port().unwrap(),
                Arc::new(Mutex::new(rx_job.subscribe())),
                Arc::clone(&server),
            );

            jobs.push(new_job);
            let _ = socket_lock.send(Message::Text(format!("Listener `{url}` added."))).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "delete" => {
            let target = args[2].to_string();

            let mut jobs = server_lock.jobs.lock().await;
            let mut jobs_owned = jobs.to_owned();

            let job = match find_job(&mut jobs_owned, target.to_owned()).await {
                Some(j) => j,
                None => {
                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                }
            };

            if !job.running {
                job.handle.lock().await.abort();
                jobs.remove(job.id as usize);
                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` deleted."))).await;
            } else {
                let _ = socket_lock.send(
                    Message::Text(format!("Listener `{target}` cannot be deleted because it's running. Please stop it before deleting."))
                ).await;
            }

            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "start" => {
            let target = args[2].to_owned();

            let mut jobs = server_lock.jobs.lock().await;
            let job = match find_job(&mut jobs, target.to_owned()).await {
                Some(j) => j,
                None => {
                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                }
            };

            if !job.running {
                let _ = server_lock.tx_job.lock().await.send(JobMessage::Start(job.id));
                job.running = true;
                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` started."))).await;
            } else {
                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is alread running"))).await;
            }
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "stop" => {
            let target = args[2].to_string();

            let mut jobs = server_lock.jobs.lock().await;
            let job = match find_job(&mut jobs, target.to_owned()).await {
                Some(j) => j,
                None => {
                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                }
            };

            if job.running {
                let _ = server_lock.tx_job.lock().await.send(JobMessage::Stop(job.id));
                job.running = false;
                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` stopped."))).await;
            } else {
                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is already stopped."))).await;
            }
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "list" => {
            let mut jobs = server_lock.jobs.lock().await;
            let output = format_listeners(&mut jobs);
            let _ = socket_lock.send(Message::Text(output)).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        _ => {
            let _ = socket_lock.send(Message::Text("Unknown arguments".to_owned())).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
            return;
        }
    }
}