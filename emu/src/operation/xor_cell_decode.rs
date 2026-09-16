const CELL: usize = 8;

pub fn xor_cell_decode(cell: &[u8; CELL]) -> u32 {
    let mut plain = [0u8; 4];

    for i in 0..4 {
        plain[i] = cell[i] ^ cell[CELL - 1 - i];
    }

    u32::from_le_bytes(plain)
}
