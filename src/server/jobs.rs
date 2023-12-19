use colored::Colorize;
use log::info;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
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

    pub receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
}

impl Job {
    pub fn new(
        id: u32,
        name: String,
        protocol: String,
        host: String,
        port: u16,
        receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
    ) -> Self {
        let mut running = false;

        let receiver_clone = Arc::clone(&receiver);

        tokio::spawn(async move {
            let mut receiver = receiver_clone.lock().await;
            loop {
                if let Ok(msg) = receiver.recv().await {
                    match(msg) {
                        JobMessage::Start(job_id) => {
                            if !running {
                                if job_id == id {
                                    info!("JobMessage: Start a listener.");
                                    // TODO: Add a method to start a listener.
                                    // ...
                                    // ...
                                    running = true;
                                }
                            } else {
                                info!("JobMessage: Listener has already started.")
                            }
                        },
                        JobMessage::Stop(job_id) => {
                            if running {
                                if job_id == id {
                                    info!("JobMessage: Stop a listener.");
                                    // TODO: Add a method to stop a listener.
                                    // ...
                                    // ...
                                    running = false;
                                }
                            } else {
                                info!("JobMessage:Listener is already stopped.")
                            }
                        },
                        JobMessage::Delete(id) => {},
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
            running,
            receiver,
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

pub fn format_jobs(jobs: &Vec<Job>) -> String  {
    info!("Getting jobs status...");
    if jobs.len() == 0 {
        return String::from("No jobs found.");
    }

    let mut output = format!("{:>5} | {:<20} | {:<32} | {:15}\n", "ID", "NAME", "URL", "STATUS");
    output = output + "------------------------------------------------------------------------------\n";

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
