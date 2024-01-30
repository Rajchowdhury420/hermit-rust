use rusqlite::Result;

use crate::utils::fs::get_app_dir;

mod agents;
pub use agents::{
    init_agents,
    add_agent,
    delete_agent,
    delete_all_agents,
    exists_agent,
    get_agent,
    get_all_agents,
    // update_agent,
};

mod implants;
pub use implants::{
    init_implants,
    add_implant,
    delete_implant,
    delete_all_implants,
    exists_implant,
    get_implant,
    get_all_implants,
};

mod keypair;
pub use keypair::{
    init_keypair,
    add_keypair,
    exists_keypair,
    delete_all_keypair,
    get_keypair,
    update_keypair,
};

mod listeners;
pub use listeners::{
    init_listeners,
    add_listener,
    delete_listener,
    delete_all_listeners,
    exists_listener,
    get_all_listeners
};

mod operators;
pub use operators::{
    init_operators,
    add_operator,
    delete_operator,
    delete_all_operators,
    exists_operator,
    get_operator,
    get_all_operators,
};

pub const DB_PATH: &str = "server/hermit.db";

#[derive(Debug)]
pub struct DB {
    pub path: String,
}

impl DB {
    pub fn new() -> Self {
        Self {
            path: format!("{}/{}", get_app_dir(), DB_PATH.to_string()),
        }
    }

}

pub fn init_db(db_path: String) -> Result<()> {
    init_listeners(db_path.to_owned())?;
    init_agents(db_path.to_owned())?;
    init_keypair(db_path.to_owned())?;
    init_implants(db_path.to_owned())?;
    init_operators(db_path.to_owned())?;

    Ok(())
}