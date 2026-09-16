const SLOT: usize = 4;

pub fn xor_row_decode(row: &[u8], slots: usize, index: usize) -> Option<u32> {
    if index >= slots {
        return None;
    }

    let value = row.get(index * SLOT..index * SLOT + SLOT)?;
    let key = row.get(slots * SLOT..slots * SLOT + SLOT)?;

    let mut plain = [0u8; SLOT];

    for (out, (encoded, keyed)) in plain.iter_mut().zip(value.iter().zip(key)) {
        *out = encoded ^ keyed;
    }

    Some(u32::from_le_bytes(plain))
}
