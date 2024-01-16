use log::warn;
use rusqlite::{Connection, Result};

use crate::server::operators::Operator;

pub fn init_operators(db_path: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "CREATE TABLE operators (
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL,
            address TEXT NOT NULL
        )",
        ()
    )?;

    Ok(())
}

pub fn add_operator(db_path: String, operator: Operator) -> Result<()> {
    let db = match Connection::open(db_path.to_owned()) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let exists = exists_operator(
        db_path.to_owned(),
        operator.name.to_owned()
    )?;

    if exists {
        warn!("Operator already exists.");
        return Ok(())
    }

    db.execute(
        "INSERT INTO operators (
            name, address
        ) VALUES (
            ?1, ?2
        )",
        (operator.name, operator.address)
    )?;

    Ok(())
}

pub fn exists_operator(db_path: String, operator_name: String) -> Result<bool> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT * from operators WHERE id = ?1 OR name = ?2"
    )?;
    let exists = stmt.exists([operator_name.to_owned(), operator_name])?;
    Ok(exists)
}

pub fn delete_operator(db_path: String, operator_name: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute(
        "DELETE FROM operators WHERE id = ?1 OR name = ?2 OR address = ?3",
        [operator_name.to_string(), operator_name.to_string(), operator_name.to_string()],
    )?;
    
    Ok(())
}

pub fn delete_all_operators(db_path: String) -> Result<()> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    db.execute("DELETE FROM operators", [])?;

    Ok(())
}

pub fn get_operator(db_path: String, name: String) -> Result<Operator> {
    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT id, name, address FROM operators WHERE id = ?1 OR name = ?2"
    )?;
    let implant = stmt.query_row([name.to_string(), name.to_string()], |row| {
        Ok(Operator::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
        ))
    })?;

    Ok(implant)
}

pub fn get_all_operators(db_path: String) -> Result<Vec<Operator>> {
    let mut operators: Vec<Operator> = Vec::new();

    let db = match Connection::open(db_path) {
        Ok(d) => d,
        Err(e) => { 
            return Err(e);
        }
    };

    let mut stmt = db.prepare(
        "SELECT id, name, address FROM operators"
    )?;
    let operator_iter = stmt.query_map([], |row| {
        Ok(Operator::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
        ))
    })?;

    for operator in operator_iter {
        operators.push(operator.unwrap());
    }

    Ok(operators)
}