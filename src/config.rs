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

    pub fn read_file(&self, file: String) -> Result<String, std::io::Error> {
        let file = format!("{}/{}", self.app_dir.display(), file);
        let mut f = fs::File::open(file)?;
        let mut data = vec![];
        f.read_to_end(&mut data)?;

        Ok(String::from_utf8(data).unwrap())
    }

    pub fn write_file(&self, file: String, data: String) -> Result<(), std::io::Error> {
        let file = format!("{}/{}", self.app_dir.display(), file);
        let mut f = fs::File::create(file)?;
        f.write_all(data.as_bytes())?;

        Ok(())
    }

    pub fn empty_file(&self, file: String) -> Result<(), std::io::Error> {
        let file = format!("{}/{}", self.app_dir.display(), file);
        fs::OpenOptions::new().truncate(true).open(file)?;
        Ok(())
    }
}