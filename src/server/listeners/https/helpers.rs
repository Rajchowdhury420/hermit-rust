use std::io::{Error, ErrorKind};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::server::{
    crypto::aesgcm::{decipher, decode, EncMessage, vec_u8_to_u8_32},
    db,
    postdata::CipherData,
};

pub fn get_server_keypair(db_path: String) -> Result<(StaticSecret, PublicKey), Error> {
    let (encoded_my_secret, encoded_my_public) = match db::get_keypair(db_path.to_string()) {
        Ok((s, p)) => (s, p),
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, format!("Error: {}", e.to_string())));
        }
    };

    let decoded_my_secret = decode(encoded_my_secret.as_bytes());
    let decoded_my_public = decode(encoded_my_public.as_bytes());

    let my_secret = StaticSecret::from(vec_u8_to_u8_32(decoded_my_secret).unwrap());
    let my_public = PublicKey::from(vec_u8_to_u8_32(decoded_my_public).unwrap());

    Ok((my_secret, my_public))
}

pub fn decipher_agent_name(ciphertext: String, nonce: String, my_secret: StaticSecret, opp_public: PublicKey) -> Result<String, Error> {
    match decipher(
        EncMessage { ciphertext, nonce },
        my_secret,
        opp_public,
    ) {
        Ok(a) => {
            return Ok(String::from_utf8(a).unwrap());
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
}

pub fn create_cipher_message(message: String, my_secret: StaticSecret, opp_public: PublicKey) -> String {
   let cipherdata = CipherData::new(
        "".to_string(),
        message.as_bytes(),
        my_secret,
        opp_public
    );
   serde_json::to_string(&cipherdata).unwrap()
}

pub fn generate_user_agent(os: String, arch: String) -> String {
    match os.as_str() {
        "linux" => "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36".to_string(),
        "windows" | _ => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36".to_string(),
    }
}
