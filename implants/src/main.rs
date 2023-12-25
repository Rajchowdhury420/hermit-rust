pub mod core;
pub mod config;

use core::run;
use config::config::Config;

include!(concat!(env!("OUT_DIR"), "/init.rs"));

fn main() {
    let (proto, host, port, sleep) = init();
    let config = Config::new(proto.to_string(), host.to_string(), port, sleep);
    run(config).unwrap();
}
