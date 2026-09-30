use aes::Aes128;
use cbc;
use cipher::{block_padding::Pkcs7, BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, KeyInit};
use ecb;
use md5::{Digest, Md5};

use crate::pack::cryptology::PackError;

type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes128EcbDec = ecb::Decryptor<Aes128>;
type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128EcbEnc = ecb::Encryptor<Aes128>;

pub fn get_md5_key(text: &str) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();

    let mut key = [0u8; 16];
    let hex_string = hex::encode(digest);
    key.copy_from_slice(&hex_string.as_bytes()[0..16]);
    key
}

pub fn decrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?;
    Ok(decrypted_slice.to_vec())
}

pub fn decrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128EcbDec::new(key.into());
    let mut buffer = data.to_vec();
    let decrypted_slice = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?;
    Ok(decrypted_slice.to_vec())
}

pub fn encrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| PackError::EncryptionFailed)?;

    Ok(encrypted_slice.to_vec())
}

pub fn encrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let encryptor = Aes128EcbEnc::new(key.into());
    let mut buffer = data.to_vec();
    let pos = buffer.len();
    buffer.resize(pos + 16, 0);

    let encrypted_slice = encryptor
        .encrypt_padded::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| PackError::EncryptionFailed)?;

    Ok(encrypted_slice.to_vec())
}