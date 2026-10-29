mod ciphers;
mod parser;

use std::error;
use std::fmt;

use serde::Deserialize;

const HEAD_DATA: &[u8] = b"battlecatsultimate";

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    InvalidHeader,
    InvalidLength,
    InvalidKeyLength,
    DecryptionFailed,
    PaddingError,
    InvalidJson,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => write!(formatter, "Invalid BCU header signature"),
            Self::InvalidLength => write!(formatter, "Data slice is too short or malformed"),
            Self::InvalidKeyLength => write!(formatter, "Provided key must be exactly 16 bytes"),
            Self::DecryptionFailed => write!(formatter, "AES decryption failed"),
            Self::PaddingError => write!(formatter, "Block padding evaluation failed"),
            Self::InvalidJson => write!(formatter, "Decrypted JSON metadata is malformed"),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FileDescriptor {
    pub path: String,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackDescriptor {
    pub files: Vec<FileDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    pub path: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    pub metadata: PackDescriptor,
    pub files: Vec<ExtractedFile>,
}

pub fn extract_password_checksum(data: &[u8], is_multi: bool) -> Result<[u8; 16], Error> {
    let expected_header = ciphers::md5_hash(HEAD_DATA);

    let (header, remainder) = parser::take_array::<16>(data)?;
    if header != expected_header {
        return Err(Error::InvalidHeader);
    }

    if !is_multi {
        let (hash, _remainder) = parser::take_array::<16>(remainder)?;
        return Ok(hash);
    }

    let (json_size, remainder) = parser::take_u32(remainder)?;
    let (_json_data, remainder) = parser::take_bytes(remainder, json_size as usize)?;

    let (inner_header, remainder) = parser::take_array::<16>(remainder)?;
    if inner_header != expected_header {
        return Err(Error::InvalidHeader);
    }

    let (hash, _remainder) = parser::take_array::<16>(remainder)?;

    Ok(hash)
}

pub fn derive_decryption_key(password: &str) -> [u8; 16] {
    ciphers::md5_hash(password.as_bytes())
}

pub fn decrypt(data: &[u8], key: &[u8; 16]) -> Result<Pack, Error> {
    let expected_header = ciphers::md5_hash(HEAD_DATA);
    let expected_iv = ciphers::md5_hash(HEAD_DATA);

    let (header, remainder) = parser::take_array::<16>(data)?;
    if header != expected_header {
        return Err(Error::InvalidHeader);
    }

    let (_hash, remainder) = parser::take_array::<16>(remainder)?;

    let (desc_size, remainder) = parser::take_u32(remainder)?;
    let desc_size_usize = desc_size as usize;
    let enc_desc_size = ciphers::regulate(desc_size_usize);

    let (enc_desc, mut stream) = parser::take_bytes(remainder, enc_desc_size)?;

    let mut dec_desc = ciphers::decrypt_chunk(enc_desc, key, &expected_iv)?;
    dec_desc.truncate(desc_size_usize);

    let Ok(metadata) = serde_json::from_slice::<PackDescriptor>(&dec_desc) else {
        return Err(Error::InvalidJson);
    };

    let mut files = Vec::with_capacity(metadata.files.len());

    for descriptor in &metadata.files {
        let enc_size = ciphers::regulate(descriptor.size);

        let (enc_chunk, next_stream) = parser::take_bytes(stream, enc_size)?;
        stream = next_stream;

        let mut dec_chunk = ciphers::decrypt_chunk(enc_chunk, key, &expected_iv)?;
        dec_chunk.truncate(descriptor.size);

        files.push(ExtractedFile {
            path: descriptor.path.clone(),
            data: dec_chunk,
        });
    }

    Ok(Pack {
        metadata,
        files,
    })
}