use colored::Colorize;
use log::info;
use url::Url;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

use super::server::Server;
use super::listeners::http::start_http_listener;

#[derive(Clone, Debug)]
pub enum JobMessage {
    Start(u32),
    Stop(u32),
    Delete(u32),
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: u32,
    pub name: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub running: bool,

    pub handle: Arc<Mutex<JoinHandle<JobMessage>>>,
}

impl Job {
    pub fn new(
        id: u32,
        name: String,
        protocol: String,
        host: String,
        port: u16,
        rx_job: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
        server: Arc<Mutex<Server>>,
    ) -> Self {

        let (tx_listener, rx_listener) = broadcast::channel(100);
        
        let tx_listener = Arc::new(Mutex::new(tx_listener));
        let rx_listener = Arc::new(Mutex::new(rx_listener));
        
        let host_clone = host.clone();
        
        let handle = tokio::spawn(async move {
            let mut rx_job = rx_job.lock().await;
            let mut running = false;
            
            loop {
                let tx_listener_clone = Arc::clone(&tx_listener);
                let rx_listener_clone = Arc::clone(&rx_listener);
                let host_clone = host_clone.clone();
                let port_clone = port.clone();

                let server_clone = Arc::clone(&server);

                if let Ok(msg) = rx_job.recv().await {
                    match msg  {
                        JobMessage::Start(job_id) => {
                            if job_id == id {
                                if !running {
                                    running = true;
                                    tokio::spawn(async move {
                                        start_http_listener(
                                            job_id,
                                            host_clone.to_string(),
                                            port_clone,
                                            rx_listener_clone,
                                            server_clone,
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
            name,
            protocol,
            host,
            port,
            running: false,
            handle: Arc::new(Mutex::new(handle)),
        }
    }
}

pub async fn find_job(jobs: &mut Vec<Job>, target: String) -> Option<&mut Job> {
    for job in jobs.iter_mut() {
        if job.id.to_string() == target || job.name == target {
            return Some(job);
        }
    }
    None
}

pub fn check_dupl_job(jobs: &mut Vec<Job>, url: Url) -> Result<(), std::io::ErrorKind> {
    let proto = url.scheme().to_string();
    let host = url.host().unwrap().to_string();
    let port = url.port().unwrap().to_owned();

    for job in jobs.iter() {
        if job.protocol == proto && job.host == host && job.port == port {
            return Err(std::io::ErrorKind::AlreadyExists);
        }
    }
    Ok(())
}

pub fn format_listeners(jobs: &Vec<Job>) -> String  {
    info!("Getting listeners status...");
    if jobs.len() == 0 {
        return String::from("No listeners found.");
    }

    let mut output = format!("{:>5} | {:<20} | {:<32} | {:15}\n", "ID", "NAME", "URL", "STATUS");
    output = output + "-".repeat(96).as_str() + "\n";

    for job in jobs {
        output = output + format!("{:>5} | {:<20} | {:<32} | {:15}\n",
            job.id.to_string(),
            job.name.to_string(),
            format!("{}://{}:{}/",
                job.protocol.to_string(),
                job.host.to_string(),
                job.port.to_string()),
            if job.running == true { "active".to_string().green().bold() } else { "inactive".to_string().red().bold() },
        ).as_str();
    }

    return output;
}
