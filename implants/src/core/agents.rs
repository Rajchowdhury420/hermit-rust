use serde::{Deserialize, Serialize};

use crate::crypto::aesgcm::{decode_decrypt, encrypt_encode};

#[derive(Clone, Deserialize, Serialize)]
pub struct AgentData {
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub listener_url: String,

    pub key: String,
    pub nonce: String,

    pub task_result: Option<Vec<u8>>,
}

impl AgentData {
    pub fn new(
        name: String,
        hostname: String,
        os: String,
        arch: String,
        listener_url: String,
        key: String,
        nonce: String
    ) -> Self {
        Self {
            name,
            hostname,
            os,
            arch,
            listener_url,
            key,
            nonce,
            task_result: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AgentDataEnc {
    pub a: String, // name
    pub b: String, // hostname
    pub c: String, // os
    pub d: String, // arch
    pub e: String, // listener_url
    
    pub f: String, // key
    pub g: String, // nonce

    pub h: String, // task_result
}

pub fn enc_agentdata(ad: AgentData) -> AgentDataEnc {
    let task_result_enc = match ad.task_result {
        Some(tr) => {
            match String::from_utf8(tr.to_vec()) {
                Ok(tr_s) => encrypt_encode(tr_s.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
                // Error will occur when the result is the image data of the screenshot.
                Err(_) => encrypt_encode(&tr, ad.key.as_bytes(), ad.nonce.as_bytes()),
            }
        }
        None => encrypt_encode(String::from("none").as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
    };

    AgentDataEnc {
        a: encrypt_encode(ad.name.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
        b: encrypt_encode(ad.hostname.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
        c: encrypt_encode(ad.os.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
        d: encrypt_encode(ad.arch.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
        e: encrypt_encode(ad.listener_url.as_bytes(), ad.key.as_bytes(), ad.nonce.as_bytes()),
        f: ad.key.clone(),
        g: ad.nonce.clone(),
        h: task_result_enc,
    }
}