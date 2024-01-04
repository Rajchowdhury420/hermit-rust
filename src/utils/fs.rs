use std::fs::{self, File};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::Path;

pub fn get_app_dir() -> String {
    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => {
            return format!("{}/.hermit", path.to_string_lossy().to_string());
        }
        _ => {
            // If the home directory not found, use the current working directory.
            return ".hermit".to_string();
        }
    }
}

pub fn mkdir(dirpath: String) -> Result<(), std::io::Error> {
    if exists_file(dirpath.to_owned()) {
        return Ok(());
    }

    let hermit_dirpath = format!("{}/{}", get_app_dir(), dirpath);

    match fs::create_dir_all(&hermit_dirpath) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn mkfile(filepath: String) -> Result<(), std::io::Error> {
    if exists_file(filepath.to_owned()) {
        return Ok(());
    }

    let hermit_filepath = format!("{}/{}", get_app_dir(), filepath);

    match fs::File::create(hermit_filepath) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn read_file(filepath: String) -> Result<Vec<u8>, Error> {
    let hermit_filepath = format!("{}/{}", get_app_dir(), filepath);

    let mut f = File::open(hermit_filepath)?;
    let mut data = vec![];
    f.read_to_end(&mut data)?;

    Ok(data)
}

pub fn write_file(filepath: String, data: &[u8]) -> Result<(), Error> {
    let hermit_filepath = format!("{}/{}", get_app_dir(), filepath);

    let mut f = File::create(hermit_filepath)?;
    f.write_all(data)?;

    Ok(())
}

pub fn empty_file(filepath: String) -> Result<(), Error> {
    let hermit_filepath = format!("{}/{}", get_app_dir(), filepath);
    
    let mut f = File::create(hermit_filepath)?;
    f.write_all(b"")?;
    Ok(())
}

pub fn exists_file(filepath: String) -> bool {
    let hermit_filepath = format!("{}/{}", get_app_dir(), filepath);

    Path::new(&hermit_filepath).exists()
}