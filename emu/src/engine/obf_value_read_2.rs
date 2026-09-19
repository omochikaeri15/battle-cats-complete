pub fn obf_value_read_2(cell: &[u8; 8]) -> u32 {
    let low = (cell[7] ^ cell[0]) as u32;
    let second = ((cell[6] ^ cell[1]) as u32) << 8;
    let third = ((cell[5] ^ cell[2]) as u32) << 16;
    let high = ((cell[4] ^ cell[3]) as u32) << 24;

    low | second | third | high
}
