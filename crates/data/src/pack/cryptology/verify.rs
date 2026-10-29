/// Checks decrypted data against the signature its filename implies.
///
/// # Arguments
/// * `data` - The decrypted bytes to check.
/// * `filename` - The expected filename, whose extension selects the signature.
///
/// # Returns
/// A `bool` that is `true` when the data matches, and `true` for extensions
/// carrying no known signature.
pub fn check_integrity(data: &[u8], filename: &str) -> bool {
    let ext = filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase());

    match ext.as_deref() {
        Some("png") => data.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        Some("csv" | "tsv" | "list" | "json" | "maanim" | "mamodel" | "imgcut") => {
            std::str::from_utf8(data).is_ok()
        }
        _ => true,
    }
}