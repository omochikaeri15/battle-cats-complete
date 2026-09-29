use sha2::{Digest, Sha256};

const HASH_BYTES: usize = 8;

pub(crate) fn hash(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            hasher.update([0u8]);
        }
        hasher.update(part.as_bytes());
    }

    hasher
        .finalize()
        .iter()
        .take(HASH_BYTES)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
