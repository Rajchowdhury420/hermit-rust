use chrono::NaiveDate;
use log::info;
use pad::Alignment;

use crate::utils::str::{
    table_format,
    TableItem,
};


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

    let mut output = String::from("\n\n");
    output = output + format!("{:<12} : {}\n", "ID", agent.id).as_str();
    output = output + format!("{:<12} : {}\n", "NAME", agent.name).as_str();
    output = output + format!("{:<12} : {}\n", "HOSTNAME", agent.hostname).as_str();
    output = output + format!("{:<12} : {}\n", "OS", format!("{}/{}", agent.os.to_owned(), agent.arch.to_owned())).as_str();
    output = output + format!("{:<12} : {}\n", "LISTENER", agent.listener_url).as_str();
    output = output + format!("{:<12} : {}\n", "PUBLIC KEY", agent.public_key).as_str();
    output = output + format!("{:<12} : {}\n", "REGISTERED", agent.registered.to_string()).as_str();
    output = output + format!("{:<12} : {}", "LAST COMMIT", agent.last_commit.to_string()).as_str();
    output
}

pub fn format_all_agents(agents: &Vec<Agent>) -> String  {
    info!("Getting agents status...");
    if agents.len() == 0 {
        return "Agents are empty".to_string();
    }

    let columns = vec![
        TableItem::new("ID".to_string(), 3, Alignment::Right, None),
        TableItem::new("NAME".to_string(), 12, Alignment::Left, None),
        TableItem::new("HOSTNAME".to_string(), 12, Alignment::Left, None),
        TableItem::new("OS".to_string(), 12, Alignment::Left, None),
        TableItem::new("LISTENER".to_string(), 20, Alignment::Left, None),
        TableItem::new("LAST COMMIT".to_string(), 13, Alignment::Left, None),
    ];
    let mut rows: Vec<Vec<TableItem>> = Vec::new();
    for agent in agents {
        let row = vec![
            TableItem::new(agent.id.to_string(), 3, Alignment::Right, None),
            TableItem::new(agent.name.to_string(), 12, Alignment::Left, None),
            TableItem::new(agent.hostname.to_string(), 12, Alignment::Left, None),
            TableItem::new(
                format!("{}/{}", agent.os.to_string(), agent.arch.to_string()), 12, Alignment::Left, None),
            TableItem::new(agent.listener_url.to_string(), 20, Alignment::Left, None),
            TableItem::new(agent.last_commit.to_string(), 13, Alignment::Left, None),
        ];
        rows.push(row);
    }
    table_format(columns, rows)
}