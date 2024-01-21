use chrono::NaiveDate;
use log::info;

use crate::utils::str::truncated_format;


#[derive(Clone, Debug)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub listener_url: String,
    pub public_key: String, // HEX encoded

    pub registered: NaiveDate,
    pub last_commit: NaiveDate,
}

impl Agent {
    pub fn new(
        id: u32,
        name: String,
        hostname: String,
        os: String,
        arch: String,
        listener_url: String,
        public_key: String,
        registered: NaiveDate,
        last_commit: NaiveDate,
    ) -> Self {
        Self {
            id,
            name,
            hostname,
            os,
            arch,
            listener_url,
            public_key,
            registered,
            last_commit,
        }
    }
}

pub fn format_agent_details(agent: Agent) -> String {
    info!("Getting the agent details...");

    let mut output = String::new();
    output = output + "\n";
    output = output + format!("{:<15} : {:<20}\n", "ID", agent.id).as_str();
    output = output + format!("{:<15} : {:<20}\n", "NAME", agent.name).as_str();
    output = output + format!("{:<15} : {:<20}\n", "HOSTNAME", agent.hostname).as_str();
    output = output + format!("{:<15} : {:<20}\n", "OS", format!("{}/{}", agent.os.to_owned(), agent.arch.to_owned())).as_str();
    output = output + format!("{:<15} : {:<20}\n", "LISTENER", agent.listener_url).as_str();
    output = output + format!("{:<15} : {:<20}\n", "PUBLIC KEY", agent.public_key).as_str();
    output = output + format!("{:<15} : {:<20}\n", "REGISTERED", agent.registered.to_string()).as_str();
    output = output + format!("{:<15} : {:<20}\n", "LAST COMMIT", agent.last_commit.to_string()).as_str();
    output
}

pub fn format_all_agents(agents: &Vec<Agent>) -> String  {
    info!("Getting agents status...");
    if agents.len() == 0 {
        return "Agents are empty".to_string();
    }

    let mut output = String::new();
    output = output + "\n";
    output = output + format!(
        "{:>3} | {:<18} | {:<15} | {:<15} | {:<25} | {:<15}\n",
        "ID", "NAME", "HOSTNAME", "OS", "LISTENER", "LAST COMMIT"
    ).as_str();
    let output_len = output.len();
    output = output + "-".repeat(output_len).as_str() + "\n";

    for agent in agents {
        output = output + format!(
            "{:>3} | {:<18} | {:<15} | {:<15} | {:<25} | {:<15}\n",
            agent.id.to_owned(),
            truncated_format(agent.name.to_owned(), 15),
            truncated_format(agent.hostname.to_owned(), 12),
            format!("{}/{}", agent.os.to_owned(), agent.arch.to_owned()),
            truncated_format(agent.listener_url.to_owned(), 22),
            agent.last_commit.to_string(),
        ).as_str();
    }

    output
}