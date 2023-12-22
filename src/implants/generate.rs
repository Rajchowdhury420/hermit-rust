use log::info;
use std::fs::File;
use std::io::{BufReader, Bytes, Read};

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

    let opt_level = 3;
    let host = "x86_64-unknown-linux-gnu";

    let target = match (os.as_str(), arch.as_str()) {
        ("linux",   "amd64")    => { "x86_64-unknown-linux-gnu"     },
        ("windows", "amd64")    => { "x86_64-unknown-windows-gnu"   },
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        },
    };

    let ext = match format.as_str() {
        // "aspx" => ".aspx",
        // "docx" => ".docx",
        "elf" => "",
        "exe" => ".exe",
        _ => {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    let input = "implants/cpp/hello.cpp";
    let output = format!("{}/server/implants/{}{}", config.app_dir.display(), name, ext);

    let mut cc = cc::Build::new();
    cc.debug(false);
    cc.cpp(true);
    // To show the list of target, run `rustc --print target-list`
    cc.target(target);
    cc.host(host);
    cc.opt_level(opt_level);
    // cc.include("implant/cpp/");

    let mut cmd = cc.get_compiler().to_command();
    cmd.args([
        input,
        "-o",
        output.as_str(),
    ]);

    match cmd.status() {
        Ok(_) => {
            let buf = BufReader::new(File::open(output.to_owned()).unwrap());
            Ok((output, buf.buffer().to_vec()))
        },
        Err(e) => Err(e),
    }
}