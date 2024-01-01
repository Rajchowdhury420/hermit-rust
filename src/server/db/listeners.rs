use rusqlite::{Connection, Result};
use log::warn;

use crate::server::listeners::listener::Listener;

pub fn init_listeners(db_path: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "CREATE TABLE listeners (
            name        TEXT NOT NULL,
            protocol    TEXT NOT NULL,
            host        TEXT NOT NULL,
            port        INTEGER NOT NULL
        )",
        (),
    )?;

    Ok(())
}

pub fn add_listener(db_path: String, listener: &Listener) -> Result<()> {
    let db = match Connection::open(db_path.to_owned()) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    // Check if already exists
    let exists = exists_listener(
        db_path.to_owned(), 
        listener.clone(),
    )?;
    
    if exists {
        warn!("Listener already exists in database.");
        return Ok(());
    }

    db.execute(
        "INSERT INTO listeners (name, protocol, host, port) VALUES (?1, ?2, ?3, ?4)",
        (
            listener.name.to_owned(),
            listener.protocol.to_owned(),
            listener.host.to_owned(),
            listener.port.to_owned()
        ),
    )?;

    Ok(())
}

pub fn exists_listener(db_path: String, listener: Listener) -> Result<bool> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT * FROM listeners WHERE protocol = ?1 AND host = ?2 AND port = ?3",
    )?;
    let exists = stmt.exists(
        [listener.protocol, listener.host, listener.port.to_string()]
    )?;
    
    Ok(exists)
}

pub fn delete_listener(db_path: String, listener_name: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "DELETE FROM listeners WHERE name = ?1",
        [listener_name],
    )?;

    Ok(())
}

pub fn get_all_listeners(db_path: String) -> Result<Vec<Listener>> {
    let mut listeners: Vec<Listener> = Vec::new();

    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT name, protocol, host, port FROM listeners"
    )?;
    let listener_iter = stmt.query_map([], |row| {
        Ok(Listener::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    })?;

    for listener in listener_iter {
        listeners.push(listener.unwrap());
    }

    Ok(listeners)
}