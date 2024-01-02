use log::info;
use serde::{Deserialize, Serialize};

use super::crypto::aesgcm::decode_decrypt;

#[derive(Deserialize)]
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

#[derive(Deserialize, Serialize)]
pub struct AgentDataEnc {
    pub a: String, // name
    pub b: String, // hostname
    pub c: String, // os
    pub d: String, // arch
    pub e: String, // listener_url
    
    pub f: String, // key
    pub g: String, // nonce

    pub h: String, // task_result
}


pub fn dec_agentdataenc(ade: AgentDataEnc) -> AgentData {
    // Decode and decrypt
    let key = ade.f.clone();
    let nonce = ade.g.clone();

    let name = decode_decrypt(ade.a.as_bytes(), key.as_bytes(), nonce.as_bytes());
    let hostname = decode_decrypt(ade.b.as_bytes(), key.as_bytes(), nonce.as_bytes());
    let os = decode_decrypt(ade.c.as_bytes(), key.as_bytes(), nonce.as_bytes());
    let arch = decode_decrypt(ade.d.as_bytes(), key.as_bytes(), nonce.as_bytes());
    let listener_url = decode_decrypt(ade.e.as_bytes(), key.as_bytes(), nonce.as_bytes());
    let task_result_tmp = decode_decrypt(ade.h.as_bytes(), key.as_bytes(), nonce.as_bytes());

    let task_result: Option<Vec<u8>> = match String::from_utf8(task_result_tmp.clone()) {
        Ok(tr_string) => {
            match tr_string.as_str() {
                "none" => None,
                _ => Some(tr_string.as_bytes().to_vec()),
            }
        }
        Err(_) => Some(task_result_tmp),
    };

    AgentData {
        name: String::from_utf8(name).unwrap(),
        hostname: String::from_utf8(hostname).unwrap(),
        os: String::from_utf8(os).unwrap(),
        arch: String::from_utf8(arch).unwrap(),
        listener_url: String::from_utf8(listener_url).unwrap(),
        key: key.to_owned(),
        nonce: nonce.to_owned(),
        task_result,
    }
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

    pub key: String,
    pub nonce: String,

    pub task: AgentTask,
    pub task_result: Option<Vec<u8>>,
}

impl Agent {
    pub fn new(
        id: u32,
        name: String,
        hostname: String,
        os: String,
        arch: String,
        listener_url: String,
        key: String,
        nonce: String,
    ) -> Self {
        Self {
            id,
            name,
            hostname,
            os,
            arch,
            listener_url,
            key,
            nonce,
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
        "{:>5} | {:<20} | {:<20} | {:<15} | {:<20} | {:<20} | {:<20}\n",
        "ID", "NAME", "HOSTNAME", "OS", "LISTENER", "KEY", "NONCE",
    );
    output = output + "-".repeat(128).as_str() + "\n";

    for agent in agents {
        output = output + format!(
            "{:>5} | {:<20} | {:<20} | {:<15} | {:<20} | {:<20} | {:<20}\n",
            agent.id.to_owned(),
            agent.name.to_owned(),
            agent.hostname.to_owned(),
            format!("{}/{}", agent.os.to_owned(), agent.arch.to_owned()),
            agent.listener_url.to_owned(),
            agent.key.to_owned(),
            agent.nonce.to_owned(),
        ).as_str();
    }

    return output;
}