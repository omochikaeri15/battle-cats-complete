use super::{Decrypted, Keys, decrypt_chunk};

const BLOCK_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackEntry {
    pub name: String,
    pub offset: u64,
    pub size: usize,
}

impl PackEntry {
    pub const fn aligned_size(&self) -> usize {
        self.size.div_ceil(BLOCK_SIZE) * BLOCK_SIZE
    }

    pub fn extract(&self, pack: &[u8], keys: &Keys) -> Option<(Vec<u8>, Decrypted)> {
        let start = usize::try_from(self.offset).ok()?;
        let end = start.checked_add(self.aligned_size())?;
        let chunk = pack.get(start..end)?;

        let (mut plaintext, origin) = decrypt_chunk(chunk, &self.name, keys);
        plaintext.truncate(self.size);

        Some((plaintext, origin))
    }

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
