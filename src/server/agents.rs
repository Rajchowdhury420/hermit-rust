use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AgentData {
    pub name: String,
    pub hostname: String,
    pub listener_url: String,

    pub task_result: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize)]
pub enum AgentTask {
    Empty,

    Screenshot,
    Shell(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub hostname: String,
    pub listener_url: String,

    pub task: AgentTask,
    pub task_result: Option<Vec<u8>>,
}

pub fn format_agents(agents: &Vec<Agent>) -> String  {
    info!("Getting agent status...");
    if agents.len() == 0 {
        return String::from("No agents found.");
    }

    let mut output = format!(
        "{:>5} | {:<20} | {:<20} | {:<20}\n",
        "ID", "NAME", "HOSTNAME", "LISTENER",
    );
    output = output + "-".repeat(96).as_str() + "\n";

    for agent in agents {
        output = output + format!(
            "{:>5} | {:<20} | {:<20} | {:<20}\n",
            agent.id.to_owned(),
            agent.name.to_owned(),
            agent.hostname.to_owned(),
            agent.listener_url.to_owned(),
        ).as_str();
    }

    return output;
}