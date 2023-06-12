use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce // Or `Aes128GcmSiv`
};
use rand::{Rng, distributions::Alphanumeric};
use md5::compute;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum CryptionError {
    #[error("Failed to encrypt string: {0}")]
    EncryptFailed(String),
    #[error("Failed to decrypt string: {0}")]
    DecryptFailed(String),
    #[error("Could not find nonce")]
    MissingNonce,
    #[error("Missing ciphered text (this shouldn't appear!)")]
    MissingCipher
}

pub fn encrypt(text: &str, config: &Config) -> Result<(Vec<u8>, String), CryptionError> {
    let nonce_str = gen_nonce();

    let cipher = get_key_cipher(config.get_secret_key().clone());

    // 96-bits; unique per message
    let nonce = Nonce::from_slice(nonce_str.as_bytes());
    let ciphertext = match cipher.encrypt(&nonce, text.as_bytes()) {
        Ok(cipher) => cipher,
        Err(e) => return Err(CryptionError::EncryptFailed(e.to_string()))
    };

    let result = (ciphertext, nonce_str);

    Ok(result)
}

pub fn decrypt(input: (&Vec<u8>, &String), config: &Config) -> Result<String, CryptionError> {
    let ciphertext = input.0;
    let nonce_str = input.1;

    let cipher = get_key_cipher(config.get_secret_key().clone());

    // 96-bits; unique per message
    let nonce = Nonce::from_slice(nonce_str.as_bytes());
    
    let plaintext = match cipher.decrypt(&nonce, ciphertext.as_ref()) {
        Ok(cipher) => cipher,
        Err(e) => return Err(CryptionError::DecryptFailed(e.to_string()))
    };

    let result = String::from_utf8_lossy(&plaintext).to_string();

    Ok(result)
}

fn gen_nonce() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect::<String>()
}

fn get_key_cipher(key: String) -> Aes256GcmSiv {
    let digest = compute(key);
    let digested_secret = format!("{:x}", digest);

    let key = GenericArray::from_slice(digested_secret.as_bytes());
    
    Aes256GcmSiv::new(&key)
}