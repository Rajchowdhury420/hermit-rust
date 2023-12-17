use colored::Colorize;
use log::info;

use super::sessions::Session;

pub enum Status {
    Pending,
    Finished,
}

pub struct Job {
    id: u32,
    session: Session,
    cmd: String,
    resp: String,
    status: Status,
}

pub fn display_jobs(jobs: &mut Vec<Job>) -> String  {
    info!("Getting jobs status...");
    if jobs.len() == 0 {
        return String::from("No jobs found.");
    }

    let mut output = String::from("");

    for job in jobs {
        match job.status {
            Status::Pending => {
                output = format!("JOB_ID:{:<5} SESSION_ID:{:<5} TARGET:{:<18} USER:{:<10} STATUS:{:15}",
                    job.id.to_string().green().bold(),
                    job.session.id.to_string().green().bold(),
                    job.session.hostname.green().bold(),
                    job.session.user.green().bold(),
                    job.cmd.bold(),
                );
            }
            Status::Finished => {
                output = format!("JOB_ID:{:<5}  SESSION_ID:{:<5}  TARGET:{:<18}  USER:{:<10}  CMD:{:<10}  STATUS:{:15}",
                    job.id.to_string().green().bold(),
                    job.session.id.to_string().green().bold(),
                    job.session.hostname.green().bold(),
                    job.session.user.green().bold(),
                    job.cmd.bold(),
                    "Finished".to_string().green().bold(),
                );
            }
        }
    }

    return output;
}