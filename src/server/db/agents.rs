use log::warn;
use rusqlite::{Connection, Result};

use crate::server::agents::Agent;

pub fn init_agents(db_path: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "CREATE TABLE agents (
            id              INTEGER PRIMARY KEY,
            name            TEXT NOT NULL UNIQUE,
            hostname        TEXT NOT NULL,
            os              TEXT NOT NULL,
            arch            TEXT NOT NULL,
            listener_url    TEXT NOT NULL,
            public_key      TEXT NOT NULL,
            registered      TEXT NOT NULL,
            last_commit     TEXT NOT NULL
        )",
        (),
    )?;

    Ok(())
}

pub fn add_agent(db_path: String, agent: Agent) -> Result<()> {
    let db = match Connection::open(db_path.to_owned()) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    // Check if already exists
    let exists = exists_agent(
        db_path.to_owned(),
        agent.name.to_string(),
    )?;

    if exists {
        warn!("Agent already exists.");
        return Ok(())
    }

    db.execute(
        "INSERT INTO agents (
            name, hostname, os, arch, listener_url, public_key, registered, last_commit
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
        )",
        (
            agent.name.to_owned(),
            agent.hostname.to_owned(),
            agent.os.to_owned(),
            agent.arch.to_owned(),
            agent.listener_url.to_owned(),
            agent.public_key.to_owned(),
            agent.registered.to_string(),
            agent.last_commit.to_string(),
        ),
    )?;

    Ok(())
}

pub fn exists_agent(db_path: String, agent_name: String) -> Result<bool> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT * FROM agents WHERE id = ?1 OR name = ?2",
    )?;
    let exists = stmt.exists([agent_name.to_string(), agent_name.to_string()])?;

    Ok(exists)
}

pub fn delete_agent(db_path: String, agent_name: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "DELETE FROM agents WHERE id = ?1 OR name = ?2",
        [agent_name.to_string(), agent_name.to_string()],
    )?;

    Ok(())
}

pub fn get_agent(
    db_path: String,
    name: String,
) -> Result<Agent> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT id, name, hostname, os, arch, listener_url, public_key, registered, last_commit
            FROM agents WHERE name = ?1"
    )?;
    let agent = stmt.query_row([name], |row| {
        Ok(Agent::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
        ))
    })?;

    Ok(agent)
}


pub fn get_all_agents(db_path: String) -> Result<Vec<Agent>> {
    let mut agents: Vec<Agent> = Vec::new();

    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT id, name, hostname, os, arch, listener_url, public_key, registered, last_commit
            FROM agents"
    )?;
    let agent_iter = stmt.query_map([], |row| {
        Ok(Agent::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
        ))
    })?;

    for agent in agent_iter {
        agents.push(agent.unwrap());
    }

    Ok(agents)
}

// pub fn update_agent(db_path: String, agent: Agent) -> Result<()> {
//     let db = match Connection::open(db_path) {
//         Ok(d) => d,
//         Err(e) => { 
//             return Err(e);
//         }
//     };

//     db.execute(
//         "UPDATE agents SET name = ?1, public_key = ?2 WHERE name = ?3",
//         [
//             agent.name,
//             agent.public_key,
//             agent.hostname,
//             agent.os,
//             agent.arch,
//             agent.listener_url
//         ]
//     )?;

//     Ok(())
// }