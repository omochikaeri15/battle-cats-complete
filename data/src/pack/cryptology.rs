//! AES decryption and encryption of asset pack chunks and manifests.

mod ciphers;
mod manifest;
mod verify;

use std::error::Error;
use std::fmt;

use crate::common::Region;
use ciphers::{decrypt_cbc, decrypt_ecb, encrypt_cbc, encrypt_ecb, get_md5_key};

pub use manifest::PackEntry;
pub use verify::check_integrity;

/// Represents errors that can occur while decrypting or encrypting pack data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackError {
    /// A key or IV string for the named region was not valid hexadecimal.
    ///
    /// The strings are decoded verbatim, so surrounding whitespace or a
    /// trailing newline read from a configuration file will produce this.
    InvalidHexFormat(Region),
    /// A key or IV string for the named region decoded to something other than 16 bytes.
    InvalidKeyLength(Region),
    /// A standard pack was encrypted without the key and IV that mode requires.
    MissingCipherParameters,
    /// AES decryption completed but the resulting padding was not valid.
    DecryptionFailed,
    /// AES encryption failed while applying padding to the input.
    EncryptionFailed,
    /// No known manifest key produced readable text from the supplied bytes.
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

/// Selects the encryption strategy that applies to a given pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackType {
    /// A regional pack, encrypted with AES in CBC mode using a region-specific key and IV.
    Standard,
    /// A server-delivered pack, encrypted with AES in ECB mode using a fixed derived key.
    Server,
    /// An image pack, which the engine stores unencrypted.
    ImageData,
}

/// Records how a chunk was recovered by [`decrypt_chunk`].
///
/// Passthrough is ordinary behavior rather than a failure, since the engine
/// stores some formats unencrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decrypted {
    /// A regional CBC cipher matched and produced the returned plaintext.
    Regional(Region),
    /// The fixed server ECB key matched and produced the returned plaintext.
    Server,
    /// No cipher produced data matching the expected file, so the input was returned verbatim.
    ///
    /// Expected for formats stored unencrypted, and equally what wrong keys
    /// produce; this value alone does not separate the two.
    Passthrough,
}

/// A single region's AES key material.
///
/// Deliberately does not implement `Debug`, to keep key material out of logs.
#[derive(Clone)]
pub struct RegionalCipher {
    /// The region these parameters decrypt.
    pub region: Region,
    /// The 16-byte AES-128 key.
    pub key: [u8; 16],
    /// The 16-byte AES-128 initialization vector.
    pub iv: [u8; 16],
}

/// A collection of per-region AES parameters to attempt decryption with.
///
/// Deliberately does not implement `Debug`, to keep key material out of logs.
#[derive(Clone, Default)]
pub struct Keys {
    /// The regional ciphers, attempted in the order they appear.
    pub ciphers: Vec<RegionalCipher>,
}

impl Keys {
    /// Parses hex-encoded key and IV strings into a structured `Keys` instance.
    ///
    /// Each decoded value must be exactly 16 bytes, as AES-128 requires. The
    /// strings are decoded exactly as supplied, with no trimming.
    ///
    /// # Arguments
    /// * `tuples` - A slice of tuples containing the `Region`, the hex-encoded key string, and the hex-encoded IV string.
    ///
    /// # Returns
    /// A `Result` containing the populated `Keys` instance on success, or a `PackError`
    /// naming the region whose hex string was invalid or improperly sized.
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

/// Decrypts a data chunk, trying each regional cipher and then the server key.
///
/// Each attempt is checked against the expected filename, so a cipher producing
/// plausible but wrong bytes is rejected. If every attempt fails, the data is
/// assumed unencrypted and returned unchanged.
///
/// # Arguments
/// * `data` - A byte slice containing the encrypted raw chunk data.
/// * `internal_filename` - The name of the file expected within the chunk, used to verify the integrity of the decrypted data.
/// * `keys` - A reference to a `Keys` struct containing the available regional ciphers.
///
/// # Returns
/// A tuple containing the processed byte vector and a `Decrypted` value recording
/// which cipher produced it. `Decrypted::Passthrough` means the returned bytes
/// are the unmodified input.
pub fn decrypt_chunk(data: &[u8], internal_filename: &str, keys: &Keys) -> (Vec<u8>, Decrypted) {
    for cipher in &keys.ciphers {
        if let Ok(result) = decrypt_cbc(data, &cipher.key, &cipher.iv)
            && check_integrity(&result, internal_filename) {
            return (result, Decrypted::Regional(cipher.region));
        }
    }

    let server_key = get_md5_key("battlecats");
    if let Ok(result) = decrypt_ecb(data, &server_key)
        && check_integrity(&result, internal_filename) {
        return (result, Decrypted::Server);
    }

    (data.to_vec(), Decrypted::Passthrough)
}

/// Encrypts a raw data chunk according to its pack classification.
///
/// Image data passes through, server packs use a derived ECB key, and standard
/// regional packs use CBC and require an explicit key and IV.
///
/// # Arguments
/// * `data` - A byte slice containing the unencrypted raw chunk data.
/// * `pack_type` - The `PackType` determining the specific encryption strategy (Standard, Server, or ImageData).
/// * `key` - An optional reference to a 16-byte array used as the AES encryption key for `Standard` packs.
/// * `iv` - An optional reference to a 16-byte array used as the initialization vector for `Standard` packs.
///
/// # Returns
/// A `Result` containing the encrypted byte vector on success, or a `PackError` if
/// required cipher parameters are missing or the encryption engine fails.
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

/// Decrypts a list manifest using the predefined ECB keys.
///
/// The "pack" key is tried first, falling back to "battlecats" if it fails or
/// yields invalid UTF-8.
///
/// # Arguments
/// * `data` - A byte slice containing the encrypted manifest list data.
///
/// # Returns
/// A `Result` containing the decrypted manifest as a UTF-8 `String` on success, or a
/// `PackError::ListDecryptionFailed` if all decryption attempts fail or produce invalid text.
pub fn decrypt_list(data: &[u8]) -> Result<String, PackError> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key)
        && let Ok(manifest_text) = String::from_utf8(bytes) { return Ok(manifest_text); }

    Err(PackError::ListDecryptionFailed)
}

/// Encrypts a list manifest into raw bytes using the "pack" ECB cipher.
///
/// # Arguments
/// * `data` - A string slice representing the unencrypted manifest content.
///
/// # Returns
/// A `Result` containing the encrypted byte vector on success, or a `PackError`
/// if the underlying AES encryption process fails.
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
    fn test_key_error_names_region() {
        let outcome = Keys::parse(&[(Region::Tw, "not hex", IV_HEX)]);
        assert_eq!(outcome.err(), Some(PackError::InvalidHexFormat(Region::Tw)));

        let outcome = Keys::parse(&[(Region::Ko, "abcd", IV_HEX)]);
        assert_eq!(outcome.err(), Some(PackError::InvalidKeyLength(Region::Ko)));
    }
}
