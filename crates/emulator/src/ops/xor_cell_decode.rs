const CELL: usize = 8;

pub fn xor_cell_decode(cell: &[u8; CELL]) -> u32 {
    let mut plain = [0u8; 4];

    for byte in 0..4 {
        plain[byte] = cell[byte] ^ cell[CELL - 1 - byte];
    }

    u32::from_le_bytes(plain)
}
