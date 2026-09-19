use super::obfuscate_value;

pub fn obf_value_set(cell: &mut [u8; 8], value: u32) {
    cell[..4].copy_from_slice(&value.to_le_bytes());
    obfuscate_value(cell);
}
