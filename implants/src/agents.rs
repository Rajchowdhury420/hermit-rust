use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RegisterAgent {
    pub hostname: String,
    pub listener_url: String,
}

impl RegisterAgent {
    pub fn new(hostname: String, listener_url: String) -> Self {
        Self {
            hostname,
            listener_url,
        }
    }
}