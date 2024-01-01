use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AgentData {
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub listener_url: String,

    pub key: String,
    pub nonce: String,

    pub task_result: Option<Vec<u8>>,
}

impl AgentData {
    pub fn new(
        name: String,
        hostname: String,
        os: String,
        arch: String,
        listener_url: String,
        key: String,
        nonce: String
    ) -> Self {
        Self {
            name,
            hostname,
            os,
            arch,
            listener_url,
            key,
            nonce,
            task_result: None,
        }
    }
}