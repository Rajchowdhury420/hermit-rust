pub mod core;

use core::run;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

fn main() {
    let (proto, host, port) = config();
    run(proto, host, port).unwrap();
}
