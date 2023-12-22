use log::{error, info};
use std::fs;
use std::path::PathBuf;

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
            Ok(_) => return {
                info!("Created `{}` directory successfully.", new_dir.to_string());
                Ok(())
            },
            Err(e) => {
                error!("Could not create `{}` directory: {}", new_dir.to_string(), e);
                return Err(e);
            },
        };
    }
}