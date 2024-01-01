use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AgentData {
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
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
    pub os: String,
    pub arch: String,
    pub listener_url: String,

    pub task: AgentTask,
    pub task_result: Option<Vec<u8>>,
}

impl Agent {
    pub fn new(id: u32, name: String, hostname: String, os: String, arch: String, listener_url: String) -> Self {
        Self {
            id,
            name,
            hostname,
            os,
            arch,
            listener_url,
            task: AgentTask::Empty,
            task_result: None,
        }
    }
}

pub fn format_agents(agents: &Vec<Agent>) -> String  {
    info!("Getting agent status...");
    if agents.len() == 0 {
        return String::from("No agents found.");
    }

    let mut output = format!(
        "{:>5} | {:<20} | {:<20} | {:<15} | {:<20}\n",
        "ID", "NAME", "HOSTNAME", "OS", "LISTENER",
    );
    output = output + "-".repeat(96).as_str() + "\n";

    for agent in agents {
        output = output + format!(
            "{:>5} | {:<20} | {:<20} | {:<15} | {:<20}\n",
            agent.id.to_owned(),
            agent.name.to_owned(),
            agent.hostname.to_owned(),
            format!("{}/{}", agent.os.to_owned(), agent.arch.to_owned()),
            agent.listener_url.to_owned(),
        ).as_str();
    }

    return output;
}