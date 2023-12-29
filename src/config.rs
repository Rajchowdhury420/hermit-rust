use log::{error, info};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Config {
    pub app_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Self {
            app_dir: PathBuf::from(".hermit"),
        }
    }

    pub fn mkdir(&self, new_dir: String) -> Result<(), std::io::Error> {
        let new_dir = format!("{}/{}", self.app_dir.display(), new_dir);
        match fs::create_dir_all(&new_dir) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn mkfile(&self, new_file: String) -> Result<(), std::io::Error> {
        let new_file = format!("{}/{}", self.app_dir.display(), new_file);
        match fs::File::create(new_file) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}