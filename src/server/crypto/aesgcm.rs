use aes_gcm_siv::{
    aead::{Aead, generic_array::GenericArray, KeyInit, Result},
    Aes256GcmSiv, Nonce,
};
use data_encoding::HEXLOWER;

use crate::utils::random::random_string;

pub fn init() -> Result<(String, String)> {
    let key = random_string(32);
    let nonce = random_string(12);

    Ok((key, nonce))
}

pub fn encrypt(plaintext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let key = GenericArray::from_slice(key);
    let nonce = Nonce::from_slice(nonce);

    let cipher = Aes256GcmSiv::new(key);
    cipher.encrypt(nonce, plaintext.as_ref())
}

pub fn decrypt(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let key = GenericArray::from_slice(key);
    let nonce = Nonce::from_slice(nonce);

    let cipher = Aes256GcmSiv::new(key);
    cipher.decrypt(nonce, ciphertext.as_ref())
}

pub fn encode(input: &[u8]) -> String {
    HEXLOWER.encode(input)
}

pub fn decode(input: &[u8]) -> Vec<u8> {
    HEXLOWER.decode(input.as_ref()).unwrap()
}