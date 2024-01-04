use colored::Colorize;
use log::info;
use std::{
    sync::Arc
};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

use super::listeners::{
    http::start_http_listener,
    https::start_https_listener,
    listener::Listener,
};

#[derive(Clone, Debug)]
pub enum JobMessage {
    Start(u32),
    Stop(u32),
    Delete(u32),
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: u32,
    pub listener: Listener,
    pub running: bool,

    pub handle: Arc<Mutex<JoinHandle<JobMessage>>>,
}

impl Job {
    pub fn new(
        id: u32,
        listener: Listener,
        rx_job: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
        db_path: String,
    ) -> Self {

        let (tx_listener, rx_listener) = broadcast::channel(100);
        
        let tx_listener = Arc::new(Mutex::new(tx_listener));
        let rx_listener = Arc::new(Mutex::new(rx_listener));
        
        let name_clone = listener.name.clone();
        let host_clone = listener.host.clone();
        let db_path_clone = db_path.clone();
        
        let handle = tokio::spawn(async move {
            let mut rx_job = rx_job.lock().await;
            let mut running = false;
            
            loop {
                let tx_listener_clone = Arc::clone(&tx_listener);
                let rx_listener_clone = Arc::clone(&rx_listener);
                let name_clone = name_clone.clone();
                let host_clone = host_clone.clone();
                let port_clone = listener.port.clone();
                let db_path_clone = db_path_clone.clone();

                if let Ok(msg) = rx_job.recv().await {
                    match msg  {
                        JobMessage::Start(job_id) => {
                            if job_id == id {
                                if !running {
                                    running = true;
                                    tokio::spawn(async move {
                                        // start_http_listener(
                                        //     job_id,
                                        //     host_clone.to_string(),
                                        //     port_clone,
                                        //     rx_listener_clone,
                                        //     db_path_clone,
                                        // ).await;
                                        start_https_listener(
                                            job_id,
                                            name_clone.to_string(),
                                            host_clone.to_string(),
                                            port_clone,
                                            rx_listener_clone,
                                            db_path_clone,
                                        ).await;
                                    });
                                } else {
                                    info!("JobMessage: Listener has already started.");
                                }
                            }
                        },
                        JobMessage::Stop(job_id) => {
                            if job_id == id {
                                if running {
                                    running = false;
                                    let _ = tx_listener_clone.lock().await.send(JobMessage::Stop(job_id));
                                    // break;
                                } else {
                                    info!("JobMessage: Listener is already stopped.");
                                }
                            }
                        },
                        JobMessage::Delete(_id) => {},
                    }
                }
            }
        });

        Self {
            id,
            listener: listener.clone(),
            running: false,
            handle: Arc::new(Mutex::new(handle)),
        }
    }
}

pub async fn find_job(jobs: &mut Vec<Job>, target: String) -> Option<&mut Job> {
    for job in jobs.iter_mut() {
        if job.id.to_string() == target || job.listener.name == target {
            return Some(job);
        }
    }
    None
}

pub fn format_listeners(jobs: &Vec<Job>) -> String  {
    info!("Getting listeners status...");
    if jobs.len() == 0 {
        return String::from("No listeners found.");
    }

    let mut output = format!("{:>5} | {:<20} | {:<20} | {:<32} | {:15}\n", "ID", "NAME", "HOSTS", "URL", "STATUS");
    output = output + "-".repeat(100).as_str() + "\n";

    for job in jobs {
        output = output + format!("{:>5} | {:<20} | {:<20} | {:<32} | {:15}\n",
            job.id.to_string(),
            job.listener.name.to_string(),
            job.listener.hostnames.to_owned().join(","),
            format!("{}://{}:{}/",
                job.listener.protocol.to_string(),
                job.listener.host.to_string(),
                job.listener.port.to_string()),
            if job.running == true { "active".to_string().green() } else { "inactive".to_string().red() },
        ).as_str();
    }

    return output;
}
