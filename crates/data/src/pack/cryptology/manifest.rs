//! The index a `.list` manifest holds once decrypted.
//!
//! A manifest opens with the number of files the pack beside it holds and then
//! names one file per line, giving the offset it begins at and the length it
//! occupies. The engine writes each file into the pack at a length rounded up to
//! the cipher's block size, so a reader takes the rounded length and cuts the
//! plaintext back to the declared one.

use super::{Decrypted, Keys, decrypt_chunk};

/// The block size the engine pads every stored file up to.
const BLOCK_SIZE: usize = 16;

/// One file's position within the pack the manifest indexes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackEntry {
    /// The name of the file, which is also the name its chunk is verified against.
    pub name: String,
    /// The byte offset the file's chunk begins at within the pack.
    pub offset: u64,
    /// The length of the file itself, before the padding the pack stores it with.
    pub size: usize,
}

impl PackEntry {
    /// The length the file's chunk actually occupies in the pack.
    ///
    /// # Returns
    /// A `usize` holding [`PackEntry::size`] rounded up to the cipher's block size.
    pub const fn aligned_size(&self) -> usize {
        self.size.div_ceil(BLOCK_SIZE) * BLOCK_SIZE
    }

    /// Recovers this file's contents out of the pack it is indexed in.
    ///
    /// The padded chunk is decrypted whole and the plaintext then cut back to
    /// the declared length, since the padding is not part of the file.
    ///
    /// # Arguments
    /// * `pack` - The raw bytes of the pack the manifest indexes.
    /// * `keys` - The regional ciphers to attempt, as `decrypt_chunk` uses them.
    ///
    /// # Returns
    /// An `Option` holding the file's bytes and the cipher that produced them,
    /// or `None` when the pack is too short to hold the chunk the entry names.
    pub fn extract(&self, pack: &[u8], keys: &Keys) -> Option<(Vec<u8>, Decrypted)> {
        let start = usize::try_from(self.offset).ok()?;
        let end = start.checked_add(self.aligned_size())?;
        let chunk = pack.get(start..end)?;

        let (mut plaintext, origin) = decrypt_chunk(chunk, &self.name, keys);
        plaintext.truncate(self.size);

        Some((plaintext, origin))
    }

    /// Reads a decrypted manifest into one entry per file it indexes.
    ///
    /// The count the manifest opens with, and any other line that does not carry
    /// all three columns, is skipped rather than yielding an entry, so a line's
    /// position in the returned vector is not its line in the manifest.
    ///
    /// # Arguments
    /// * `manifest` - The plaintext manifest, as `decrypt_list` returns it.
    ///
    /// # Returns
    /// A `Vec` holding the entries in the order the manifest names them.
    pub fn parse<B: AsRef<[u8]>>(manifest: B) -> Vec<Self> {
        parse_inner(manifest.as_ref())
    }
}

fn parse_inner(manifest: &[u8]) -> Vec<PackEntry> {
    let text = String::from_utf8_lossy(manifest);

    text.lines().filter_map(read_entry).collect()
}

fn read_entry(line: &str) -> Option<PackEntry> {
    let mut columns = line.split(',').map(str::trim);

    let name = columns.next().filter(|name| !name.is_empty())?;
    let offset = columns.next()?.parse().ok()?;
    let size = columns.next()?.parse().ok()?;

    Some(PackEntry { name: name.to_owned(), offset, size })
}

#[cfg(test)]
mod tests {
    use super::super::{PackType, encrypt_chunk};
    use super::*;
    use crate::common::Region;

    const KEY_HEX: &str = "0123456789abcdef0123456789abcdef";
    const IV_HEX: &str = "fedcba9876543210fedcba9876543210";

    /// The opening of a real manifest, whose first line is the entry count.
    const MANIFEST: &str = "3\nunit001.csv,0,1520\nunit002.csv,1536,880\nimgcut_001.imgcut,2432,64\n";

    #[test]
    fn the_entry_count_is_not_an_entry() {
        let entries = PackEntry::parse(MANIFEST);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], PackEntry { name: "unit001.csv".into(), offset: 0, size: 1520 });
        assert_eq!(entries[2].name, "imgcut_001.imgcut");
    }

    #[test]
    fn a_line_missing_a_column_yields_no_entry() {
        let entries = PackEntry::parse("2\nunit001.csv,0\n,10,20\nunit002.csv,x,880\nunit003.csv,16,32\n");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "unit003.csv");
    }

    #[test]
    fn a_length_is_padded_up_to_the_block_size() {
        let entries = PackEntry::parse(MANIFEST);

        assert_eq!(entries[0].aligned_size(), 1520);
        assert_eq!(entries[1].aligned_size(), 880);
        assert_eq!(entries[2].aligned_size(), 64);

        let odd = PackEntry { name: "a.csv".into(), offset: 0, size: 1 };
        assert_eq!(odd.aligned_size(), 16);
    }

    #[test]
    fn extraction_cuts_the_padding_back_off() {
        let keys = Keys::parse(&[(Region::En, KEY_HEX, IV_HEX)]).unwrap();
        let cipher = &keys.ciphers[0];
        let payload = b"HP,ATK,RANGE\n100,50,250";

        let chunk = encrypt_chunk(payload, PackType::Standard, Some(&cipher.key), Some(&cipher.iv)).unwrap();

        let mut pack = vec![0_u8; 32];
        pack.extend_from_slice(&chunk);

        let entry = PackEntry { name: "unit_01.csv".into(), offset: 32, size: payload.len() };
        let (plaintext, origin) = entry.extract(&pack, &keys).unwrap();

        assert_eq!(origin, Decrypted::Regional(Region::En));
        assert_eq!(plaintext, payload);
    }

    #[test]
    fn an_entry_reaching_past_the_pack_extracts_nothing() {
        let keys = Keys::default();
        let entry = PackEntry { name: "unit_01.csv".into(), offset: 32, size: 64 };

        assert!(entry.extract(&[0_u8; 48], &keys).is_none());
    }
}
