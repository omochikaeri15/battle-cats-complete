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

fn final_block(data: &[u8]) -> Option<(usize, [u8; 16])> {
    let start = data.len().checked_sub(16).filter(|_| data.len().is_multiple_of(16))?;
    let mut block = [0u8; 16];
    block.copy_from_slice(&data[start..]);
    Some((start, block))
}

pub fn padded_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> bool {
    let Some((start, mut block)) = final_block(data) else {
        return false;
    };

    let mut chain = *iv;
    if start >= 16 {
        chain.copy_from_slice(&data[start - 16..start]);
    }

    Aes128CbcDec::new(key.into(), (&chain).into()).decrypt_padded::<Pkcs7>(&mut block).is_ok()
}

pub fn padded_ecb(data: &[u8], key: &[u8; 16]) -> bool {
    final_block(data).is_some_and(|(_, mut block)| Aes128EcbDec::new(key.into()).decrypt_padded::<Pkcs7>(&mut block).is_ok())
}

pub fn decrypt_cbc(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    let length = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?
        .len();
    buffer.truncate(length);
    Ok(buffer)
}

pub fn decrypt_ecb(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, PackError> {
    let decryptor = Aes128EcbDec::new(key.into());
    let mut buffer = data.to_vec();
    let length = decryptor
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|_| PackError::DecryptionFailed)?
        .len();
    buffer.truncate(length);
    Ok(buffer)
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