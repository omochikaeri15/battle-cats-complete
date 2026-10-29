mod ciphers;
mod manifest;
mod verify;

use std::error::Error;
use std::fmt;

use crate::common::Region;
use ciphers::{decrypt_cbc, decrypt_ecb, encrypt_cbc, encrypt_ecb, get_md5_key, padded_cbc, padded_ecb};

pub use manifest::PackEntry;
pub use verify::check_integrity;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackError {
    InvalidHexFormat(Region),
    InvalidKeyLength(Region),
    MissingCipherParameters,
    DecryptionFailed,
    EncryptionFailed,
    ListDecryptionFailed,
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexFormat(region) => {
                write!(f, "Invalid hexadecimal format in the key or IV for region {region:?}")
            }
            Self::InvalidKeyLength(region) => {
                write!(f, "Decoded key or IV for region {region:?} must be exactly 16 bytes")
            }
            Self::MissingCipherParameters => write!(f, "Required key and IV parameters were not provided"),
            Self::DecryptionFailed => write!(f, "AES decryption or padding validation failed"),
            Self::EncryptionFailed => write!(f, "AES encryption or padding application failed"),
            Self::ListDecryptionFailed => write!(f, "List manifest decryption failed (Invalid keys or corrupted file)"),
        }
    }
}

impl Error for PackError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackType {
    Standard,
    Server,
    ImageData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decrypted {
    Regional(Region),
    Server,
    Passthrough,
}

#[derive(Clone)]
pub struct RegionalCipher {
    pub region: Region,
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

#[derive(Clone, Default)]
pub struct Keys {
    pub ciphers: Vec<RegionalCipher>,
}

impl Keys {
    pub fn parse(tuples: &[(Region, &str, &str)]) -> Result<Self, PackError> {
        let mut ciphers = Vec::with_capacity(tuples.len());
        for (region, hex_key, hex_iv) in tuples {
            ciphers.push(Self::parse_cipher(*region, hex_key, hex_iv)?);
        }
        Ok(Self { ciphers })
    }

    fn parse_cipher(region: Region, hex_key: &str, hex_iv: &str) -> Result<RegionalCipher, PackError> {
        let key_bytes = hex::decode(hex_key).map_err(|_| PackError::InvalidHexFormat(region))?;
        let iv_bytes = hex::decode(hex_iv).map_err(|_| PackError::InvalidHexFormat(region))?;

        let key: [u8; 16] = key_bytes.try_into().map_err(|_| PackError::InvalidKeyLength(region))?;
        let iv: [u8; 16] = iv_bytes.try_into().map_err(|_| PackError::InvalidKeyLength(region))?;

        Ok(RegionalCipher { region, key, iv })
    }
}

pub fn decrypt_chunk(data: &[u8], internal_filename: &str, keys: &Keys) -> (Vec<u8>, Decrypted) {
    for cipher in &keys.ciphers {
        if padded_cbc(data, &cipher.key, &cipher.iv)
            && let Ok(result) = decrypt_cbc(data, &cipher.key, &cipher.iv)
            && check_integrity(&result, internal_filename) {
            return (result, Decrypted::Regional(cipher.region));
        }
    }

    let server_key = get_md5_key("battlecats");
    if padded_ecb(data, &server_key)
        && let Ok(result) = decrypt_ecb(data, &server_key)
        && check_integrity(&result, internal_filename) {
        return (result, Decrypted::Server);
    }

    (data.to_vec(), Decrypted::Passthrough)
}

pub fn encrypt_chunk(data: &[u8], pack_type: PackType, key: Option<&[u8; 16]>, iv: Option<&[u8; 16]>) -> Result<Vec<u8>, PackError> {
    match pack_type {
        PackType::ImageData => Ok(data.to_vec()),
        PackType::Server => encrypt_ecb(data, &get_md5_key("battlecats")),
        PackType::Standard => {
            let (Some(cipher_key), Some(cipher_iv)) = (key, iv) else {
                return Err(PackError::MissingCipherParameters);
            };
            encrypt_cbc(data, cipher_key, cipher_iv)
        }
    }
}

pub fn decrypt_list(data: &[u8]) -> Result<String, PackError> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    Err(PackError::ListDecryptionFailed)
}

pub fn encrypt_list(data: &str) -> Result<Vec<u8>, PackError> {
    encrypt_ecb(data.as_bytes(), &get_md5_key("pack"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Region;

    const KEY_HEX: &str = "0123456789abcdef0123456789abcdef";
    const IV_HEX: &str = "fedcba9876543210fedcba9876543210";

    #[test]
    fn test_list_manifest_roundtrip() {
        let original_manifest = "0,DataLocal.pack\n1,ui_001.png\n2,unit_01.csv";
        let encrypted_bytes = encrypt_list(original_manifest)
            .expect("Failed to encrypt synthetic manifest");
        let decrypted_manifest = decrypt_list(&encrypted_bytes)
            .expect("Failed to decrypt synthetic manifest");

        assert_eq!(original_manifest, decrypted_manifest);
    }

    #[test]
    fn test_standard_chunk_roundtrip() {
        let keys = Keys::parse(&[(Region::En, KEY_HEX, IV_HEX)])
            .expect("Failed to parse synthetic keys");
        let cipher = &keys.ciphers[0];
        let internal_filename = "unit_01.csv";
        let original_payload = "HP,ATK,RANGE\n100,50,250";

        let encrypted_chunk = encrypt_chunk(
            original_payload.as_bytes(),
            PackType::Standard,
            Some(&cipher.key),
            Some(&cipher.iv)
        ).expect("Failed to encrypt standard CBC chunk");

        let (decrypted_chunk, origin) = decrypt_chunk(&encrypted_chunk, internal_filename, &keys);

        assert_eq!(origin, Decrypted::Regional(Region::En), "Did not correctly match the EN region CBC key");
        assert_eq!(original_payload.as_bytes(), decrypted_chunk.as_slice());
    }

    #[test]
    fn test_server_chunk_roundtrip() {
        let keys = Keys::default();

        let internal_filename = "server_data.json";
        let original_payload = "{\"status\": \"ok\", \"version\": 140200}";
        let encrypted_chunk = encrypt_chunk(
            original_payload.as_bytes(),
            PackType::Server,
            None,
            None
        ).expect("Failed to encrypt server ECB chunk");

        let (decrypted_chunk, origin) = decrypt_chunk(&encrypted_chunk, internal_filename, &keys);

        assert_eq!(origin, Decrypted::Server, "Server packs should report a server origin");
        assert_eq!(original_payload.as_bytes(), decrypted_chunk.as_slice());
    }

    #[test]
    fn test_image_roundtrip() {
        let dummy_image_data: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let internal_filename = "ui_001.png";

        let encrypted = encrypt_chunk(dummy_image_data, PackType::ImageData, None, None)
            .expect("Failed image pass-through");

        let keys = Keys::default();
        let (decrypted, origin) = decrypt_chunk(&encrypted, internal_filename, &keys);

        assert_eq!(origin, Decrypted::Passthrough);
        assert_eq!(dummy_image_data, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_keys_report_passthrough() {
        let keys = Keys::parse(&[(Region::En, KEY_HEX, IV_HEX)])
            .expect("Failed to parse synthetic keys");
        let cipher = &keys.ciphers[0];
        let payload = "HP,ATK,RANGE\n100,50,250";

        let encrypted = encrypt_chunk(
            payload.as_bytes(),
            PackType::Standard,
            Some(&cipher.key),
            Some(&cipher.iv)
        ).expect("Failed to encrypt standard CBC chunk");

        let other = Keys::parse(&[(Region::Ja, IV_HEX, KEY_HEX)])
            .expect("Failed to parse synthetic keys");
        let (returned, origin) = decrypt_chunk(&encrypted, "unit_01.csv", &other);

        assert_eq!(origin, Decrypted::Passthrough);
        assert_eq!(returned.as_slice(), encrypted.as_slice());
    }

    #[test]
    fn test_later_cipher_matches_after_earlier_ones_fail() {
        let keys = Keys::parse(&[(Region::Ja, IV_HEX, KEY_HEX), (Region::En, KEY_HEX, IV_HEX)])
            .expect("Failed to parse synthetic keys");
        let cipher = &keys.ciphers[1];

        for payload in ["1,2,3".to_string(), "HP,ATK,RANGE\n100,50,250\n".repeat(8)] {
            let encrypted = encrypt_chunk(
                payload.as_bytes(),
                PackType::Standard,
                Some(&cipher.key),
                Some(&cipher.iv)
            ).expect("Failed to encrypt standard CBC chunk");

            let (decrypted, origin) = decrypt_chunk(&encrypted, "unit_01.csv", &keys);

            assert_eq!(origin, Decrypted::Regional(Region::En));
            assert_eq!(payload.as_bytes(), decrypted.as_slice());
        }
    }

    #[test]
    fn test_key_error_names_region() {
        let outcome = Keys::parse(&[(Region::Tw, "not hex", IV_HEX)]);
        assert_eq!(outcome.err(), Some(PackError::InvalidHexFormat(Region::Tw)));

        let outcome = Keys::parse(&[(Region::Ko, "abcd", IV_HEX)]);
        assert_eq!(outcome.err(), Some(PackError::InvalidKeyLength(Region::Ko)));
    }
}
