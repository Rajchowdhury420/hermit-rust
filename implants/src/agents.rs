use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AgentData {
    pub name: String,
    pub hostname: String,
    pub listener_url: String,

    pub task_result: Option<Vec<u8>>,
}

impl AgentData {
    pub fn new(name: String, hostname: String, listener_url: String) -> Self {
        Self {
            name,
            hostname,
            listener_url,
            task_result: None,
        }
    }
}