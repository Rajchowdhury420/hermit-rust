use log::info;
use std::fs::File;
use std::io::{BufReader, Bytes, Read};
use std::process::Command;

use crate::config::Config;

/// Generate an implant
/// References:
/// - https://github.com/BishopFox/sliver/blob/master/server/generate/binaries.go#L325
pub fn generate(
    config: &Config,
    name: String,
    listener_url: String,
    os: String,
    arch: String,
    format: String
) -> Result<(String, Vec<u8>), std::io::Error> {

    info!("Generating an implant...");

    // Create `implants` directory
    config.mkdir("implants".to_string()).unwrap();

    let ext = match format.as_str() {
        // "aspx" => ".aspx",
        // "docx" => ".docx",
        "elf" => "",
        "exe" => ".exe",
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    let input = "implants/cpp/messagebox.cpp";
    let output = format!("{}/server/implants/{}{}", config.app_dir.display(), name, ext);

    let (gcc, args) = match (os.as_str(), arch.as_str()) {
        ("linux",   "amd64")    => { ("gcc", [&input, "-o", &output]) }
        ("windows", "amd64")    => { ("/usr/bin/x86_64-w64-mingw32-gcc", [&input, "-o", &output]) }
        ("windows", "i386")     => { ("/usr/bin/i686-w64-mingw32-gcc", [&input, "-o", &output]) }
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    let result = Command::new(gcc)
        .args(args)
        .output();

    match result {
        Ok(_) => {
            let mut f = File::open(output.to_owned()).unwrap();
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).unwrap();
            Ok((output, buffer))
        }
        Err(e) => {
            Err(e)
        }
    }
}