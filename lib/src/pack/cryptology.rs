use crate::pack::utils::ciphers::{
    decrypt_cbc, decrypt_ecb,
    encrypt_cbc, encrypt_ecb,
    get_md5_key
};
use crate::pack::utils::verify;

/// Represents the regional version of The Battle Cats.
///
/// Cryptographic keys differ based on the target game region. This enum is used
/// to identify which key successfully decrypted a file, allowing the orchestrator
/// to track the active region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// English / Global version (`en`)
    En,
    /// Japanese version (`jp`)
    Jp,
    /// Korean version (`kr`)
    Kr,
    /// Taiwanese version (`tw`)
    Tw,
}

/// Represents the encryption strategy required for a specific type of pack file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackType {
    /// Standard game data requiring regional CBC encryption.
    Standard,
    /// Server data requiring global ECB encryption.
    Server,
    /// Image data requiring no encryption.
    ImageData,
}

/// A pure representation of the cryptographic keys required for pack decryption.
///
/// This struct holds the parsed user keys to prevent redundant hex decoding
/// during massive bulk decryption operations.
pub struct Keys {
    /// A list of tuples containing `(Key Hex String, IV Hex String, Region)`.
    pub tuples: Vec<(String, String, Region)>,
}

/// Attempts to decrypt a raw chunk of pack data using a brute-force key resolution strategy.
///
/// This function tests the provided regional keys against the cipher. If a regional key
/// succeeds, it verifies the decrypted output against known file magic bytes (e.g., PNG headers)
/// or valid UTF-8. If all regional keys fail, it automatically falls back to testing the global
/// server keys.
///
/// # Arguments
/// * `data` - The raw, encrypted byte array extracted directly from the pack.
/// * `internal_filename` - The target name of the file, used internally to verify the decrypted content type.
/// * `keys` - A `Keys` struct containing the user's regional keys.
///
/// # Returns
/// A `Result` containing a tuple of the decrypted bytes and an `Option<Region>`.
/// The `Region` indicates which regional key succeeded. If the global server key was used, the region will be `None`.
pub fn decrypt_chunk(
    data: &[u8],
    internal_filename: &str,
    keys: &Keys
) -> Result<(Vec<u8>, Option<Region>), String> {
    for (k_hex, iv_hex, region) in &keys.tuples {
        let Ok(key_bytes) = hex::decode(k_hex) else { continue; };
        let Ok(iv_bytes) = hex::decode(iv_hex) else { continue; };
        let (Ok(key_arr), Ok(iv_arr)) = (key_bytes.try_into(), iv_bytes.try_into()) else { continue; };

        if let Ok(result) = decrypt_cbc(data, &key_arr, &iv_arr) {
            if verify::is_valid(&result, internal_filename) {
                return Ok((result, Some(*region)));
            }
        }
    }

    let server_key = get_md5_key("battlecats");
    if let Ok(result) = decrypt_ecb(data, &server_key) {
        if verify::is_valid(&result, internal_filename) {
            return Ok((result, None));
        }
    }

    Ok((data.to_vec(), None))
}

/// Encrypts a raw chunk of game data based on its designated `PackType`.
///
/// This routing function executes the correct cryptographic strategy dictated by the orchestrator.
///
/// # Arguments
/// * `data` - The raw, plaintext byte array of the file to be encrypted.
/// * `pack_type` - A strict enum defining the target encryption strategy.
/// * `key` - An optional 16-byte array representing the specific regional cipher key.
/// * `iv` - An optional 16-byte array representing the specific regional initialization vector.
///
/// # Returns
/// A `Result` containing the encrypted byte array ready to be streamed into a pack buffer.
///
/// # Errors
/// Returns an error string if `PackType::Standard` is requested but the required `key` or `iv`
/// are missing, or if the underlying AES padding/encryption fails.
pub fn encrypt_chunk(
    data: &[u8],
    pack_type: PackType,
    key: Option<&[u8; 16]>,
    iv: Option<&[u8; 16]>
) -> Result<Vec<u8>, String> {
    match pack_type {
        PackType::ImageData => Ok(data.to_vec()),
        PackType::Server => {
            let server_key = get_md5_key("battlecats");
            encrypt_ecb(data, &server_key)
        },
        PackType::Standard => {
            let (Some(cipher_key), Some(cipher_iv)) = (key, iv) else {
                return Err("A Key and IV are required to encrypt standard game data.".to_string());
            };
            encrypt_cbc(data, cipher_key, cipher_iv)
        }
    }
}

/// Decrypts a `.list` file containing pack manifest data.
///
/// List files act as the table of contents for `.pack` files. They use a specific
/// ECB encryption pattern utilizing either a standard "pack" MD5 key or the
/// global server MD5 key, distinct from standard asset CBC encryption.
///
/// # Arguments
/// * `data` - The raw, encrypted bytes of the `.list` file.
///
/// # Returns
/// A `Result` containing the decrypted manifest as a valid, readable UTF-8 `String`.
///
/// # Errors
/// Returns an error string if neither the pack key nor the global server key can
/// successfully decrypt the data, or if the resulting bytes are not valid UTF-8.
pub fn decrypt_list(data: &[u8]) -> Result<String, String> {
    let pack_key = get_md5_key("pack");
    if let Ok(bytes) = decrypt_ecb(data, &pack_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    let bc_key = get_md5_key("battlecats");
    if let Ok(bytes) = decrypt_ecb(data, &bc_key) {
        if let Ok(s) = String::from_utf8(bytes) { return Ok(s); }
    }

    Err("List decryption failed (Invalid keys or corrupted file)".into())
}

/// Encrypts a string into a `.list` file format suitable for game engine packing.
///
/// # Arguments
/// * `data` - The raw, plaintext manifest string containing file entries, offsets, and sizes.
///
/// # Returns
/// A `Result` containing the fully padded and ECB-encrypted byte array ready to be written to disk.
pub fn encrypt_list(data: &str) -> Result<Vec<u8>, String> {
    let pack_key = get_md5_key("pack");
    encrypt_ecb(data.as_bytes(), &pack_key)
}