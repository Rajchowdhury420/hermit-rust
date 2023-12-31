use log::{error, info};
use std::fs::File;
use std::env;
use std::io::{Error, ErrorKind, Read};
use std::process::Command;
use std::str::from_utf8;
use url::Url;

use crate::config::Config;
use crate::utils::fs::{get_app_dir, read_file};

/// Generate an implant
/// References:
/// - https://github.com/BishopFox/sliver/blob/master/server/generate/binaries.go#L325
pub fn generate(
    config: &Config,
    name: String,
    listener_url: String,
    os: String,
    arch: String,
    format: String,
    sleep: u16,
) -> Result<(String, Vec<u8>), Error> {

    info!("Generating an implant...");

    let parsed_url = Url::parse(&listener_url).unwrap();
    let proto = parsed_url.scheme();
    let host = parsed_url.host().unwrap();
    let port = parsed_url.port().unwrap();

    // Set environment variables for `config.rs` when building an implant.
    env::set_var("LPROTO", proto.to_string());
    env::set_var("LHOST", host.to_string());
    env::set_var("LPORT", port.to_string());
    env::set_var("SLEEP", sleep.to_string());
    env::set_var("OUT_DIR", format!("implants/src"));

    let outdir = format!("{}/implants/{}", get_app_dir(), name.to_string());

    let (cmd, args, outfile) = match (os.as_str(), arch.as_str(), format.as_str()) {
        ("linux", "amd64", "elf") => {
            (
                "cargo",
                [
                    "build",
                    "--manifest-path=implants/Cargo.toml",
                    "--target",
                    "x86_64-unknown-linux-gnu",
                    "--target-dir",
                    outdir.as_str(),
                    "--release"
                ],
                format!("implants/{}/x86_64-unknown-linux-gnu/release/implant", name.to_string()),
            )
        }
        ("linux", "i686", "elf") => {
            (
                "cargo",
                [
                    "build",
                    "--manifest-path=implants/Cargo.toml",
                    "--target",
                    "i686-unknown-linux-gnu",
                    "--target-dir",
                    outdir.as_str(),
                    "--release"
                ],
                format!("implants/{}/i686-unknown-linux-gnu/release/implant", name.to_string()),
            )
        }
        ("windows", "amd64", "exe") => {
            (
                "cargo",
                [
                    "build",
                    "--manifest-path=implants/Cargo.toml",
                    "--target",
                    "x86_64-pc-windows-gnu",
                    "--target-dir",
                    outdir.as_str(),
                    "--release"
                ],
                format!("implants/{}/x86_64-pc-windows-gnu/release/implant.exe", name.to_string()),
            )
        }
        ("windows", "i686", "exe") => {
            (
                "cargo",
                [
                    "build",
                    "--manifest-path=implants/Cargo.toml",
                    "--target",
                    "i686-pc-windows-gnu",
                    "--target-dir",
                    outdir.as_str(),
                    "--release"
                ],
                format!("implants/{}/i686-pc-windows-gnu/release/implant.exe", name.to_string()),
            )
        }
        _ => {
            return Err(Error::new(ErrorKind::Other, "Invalid options."));
        }
    };

    let output = Command::new(cmd)
        .args(args)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                info!("Generation Success: {:#?}", from_utf8(&o.stdout).unwrap());
                let buffer = read_file(outfile.to_string()).unwrap();
                return Ok((outfile.to_string(), buffer));
            } else {
                error!("Generation Error: {:#?}", from_utf8(&o.stderr).unwrap());
                return Err(Error::new(ErrorKind::Other, "Failed to generate an implant."));
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            return Err(e);
        }
    }
}