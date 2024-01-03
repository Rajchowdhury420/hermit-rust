use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Read environment variables for initiali settings
    let lproto = env::var_os("HERMIT_LPROTO").unwrap();
    let lhost = env::var_os("HERMIT_LHOST").unwrap();
    let lport = env::var_os("HERMIT_LPORT").unwrap();
    let sleep = env::var_os("HERMIT_SLEEP").unwrap();
    let user_agent = env::var_os("HERMIT_USER_AGENT").unwrap();
    let server_public_key = env::var_os("HERMIT_PUBLIC_KEY").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap(); // This is not allowed the prefix `HERMIT_` by cargo.
    let dest_path = Path::new(&out_dir).join("init.rs");

    fs::write(
        &dest_path,
        format!("pub fn init() -> (
            &'static str,
            &'static str,
            u16,
            u64,
            &'static str,
            &'static str,
        ) {}
            (\"{}\", \"{}\", {}, {}, \"{}\", \"{}\")
        {}
        ",
        "{",
        lproto.into_string().unwrap(),
        lhost.into_string().unwrap(),
        lport.into_string().unwrap(),
        sleep.into_string().unwrap(),
        user_agent.into_string().unwrap(),
        server_public_key.into_string().unwrap(),
        "}"
    )).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}