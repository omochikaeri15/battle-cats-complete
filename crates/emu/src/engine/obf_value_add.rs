use super::obfuscate_value;

pub fn obf_value_add(cell: &mut [u8; 8], delta: i32) {
    let value = ((cell[7] ^ cell[0]) as u32)
        .wrapping_add(delta as u32)
        .wrapping_add(((cell[6] ^ cell[1]) as u32) << 8)
        .wrapping_add(((cell[5] ^ cell[2]) as u32) << 0x10)
        .wrapping_add(((cell[4] ^ cell[3]) as u32) << 0x18);

    cell[..4].copy_from_slice(&value.to_le_bytes());
    obfuscate_value(cell);
}
