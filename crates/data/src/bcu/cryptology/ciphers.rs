use aes::Aes128;
use block_padding::NoPadding;
use cbc::Decryptor;
use cipher::{BlockModeDecrypt, KeyIvInit};
use md5::{Digest, Md5};

use crate::bcu::cryptology::Error;

type Aes128Cbc = Decryptor<Aes128>;

pub fn md5_hash(data: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn decrypt_chunk(data: &[u8], key: &[u8; 16], initialization_vector: &[u8; 16]) -> Result<Vec<u8>, Error> {
    let mut buffer = data.to_vec();
    let decryptor = Aes128Cbc::new(key.into(), initialization_vector.into());

    let Ok(decrypted) = decryptor.decrypt_padded::<NoPadding>(&mut buffer) else {
        return Err(Error::DecryptionFailed);
    };

    Ok(decrypted.to_vec())
}

pub fn regulate(size: usize) -> usize {
    if (size & 0xF) == 0 {
        size
    } else {
        (size | 0xF) + 1
    }
}