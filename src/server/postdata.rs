use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};

use super::crypto::aesgcm::cipher;

#[derive(Deserialize)]
pub struct RegisterAgentData {
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub listener_url: String,
    pub public_key: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PlainData {
    pub p: String, // plaintext
}

impl PlainData {
    pub fn new(plaintext: String) -> Self {
        Self { p: plaintext }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CipherData {
    pub p: String, // plaintext (mainly used for the agent name)
    pub c: String, // ciphertext
    pub n: String, // nonce
}

impl CipherData {
    pub fn new(
        plaintext: String,
        plaindata_to_cipher: &[u8],
        my_secret: StaticSecret,
        opp_public: PublicKey
    ) -> Self {
        let enc = cipher(plaindata_to_cipher, my_secret, opp_public);
        Self {
            p: plaintext,
            c: enc.ciphertext,
            n: enc.nonce,
        }
    }
}