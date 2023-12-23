use log::{error, info};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
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
) -> Result<(String, Vec<u8>), Error> {

    info!("Generating an implant...");

    let ext = match format.as_str() {
        // "aspx" => ".aspx",
        // "docx" => ".docx",
        "elf" => "",
        "exe" => ".exe",
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    let infile = "implants/cpp/messagebox.cpp";
    let outfile = format!("{}/server/implants/{}{}", config.app_dir.display(), name, ext);

    let (gcc, args) = match (os.as_str(), arch.as_str()) {
        ("linux",   "amd64")    => { ("g++", [&infile, "-o", &outfile, ""]) }
        ("windows", "amd64")    => { ("/usr/bin/x86_64-w64-mingw32-g++", [&infile, "-o", &outfile, "-lwinhttp"]) }
        ("windows", "i386")     => { ("/usr/bin/i686-w64-mingw32-g++", [&infile, "-o", &outfile, "-lwinhttp"]) }
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    let output = Command::new(gcc)
        .args(args)
        .output();

    match output {
        Ok(o) => {
            info!("{:?}", o);
            if o.status.success() {
                let mut f = File::open(outfile.to_owned()).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                return Ok((outfile, buffer));
            } else {
                return Err(Error::new(ErrorKind::Other, "Failed to generate an implant."));
            }
        }
        Err(e) => {
            error!("{e}");
            return Err(e);
        }
    }
}