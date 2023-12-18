use colored::Colorize;
use log::info;

#[derive(Debug)]
pub struct Job {
    id: u32,
    name: String,
    protocol: String,
    host: String,
    port: u16,
    running: bool,
}

impl Job {
    pub fn new(
        id: u32,
        name: String,
        protocol: String,
        host: String,
        port: u16
    ) -> Self {

        Self {
            id,
            name,
            protocol,
            host,
            port,
            running: false,
        }
    }
}

pub fn format_jobs(jobs: &mut Vec<Job>) -> String  {
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