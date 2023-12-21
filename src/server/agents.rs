use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterAgent {
    pub hostname: String,
    pub listener_url: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub hostname: String,
    pub listener_url: String,
}

pub fn format_agents(agents: &Vec<Agent>) -> String  {
    info!("Getting jobs status...");
    if agents.len() == 0 {
        return String::from("No jobs found.");
    }

    let mut output = format!("{:>5} | {:<20} | {:<20} | {:<20}\n", "ID", "NAME", "HOSTNAME", "LISTENER");
    output = output + "------------------------------------------------------------------------------\n";

    for agent in agents {
        output = output + format!("{:>5} | {:<20} | {:<20} | {:<20}\n",
            agent.id.to_string(),
            agent.name.to_string(),
            agent.hostname.to_string(),
            agent.listener_url.to_string(),
        ).as_str();
    }

    return output;
}