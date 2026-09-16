use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

const KEY_SPAN: u64 = 0x100;

pub fn obfuscate_value(cell: &mut [u8; 8]) {
    cell[4] = (RandomState::new().build_hasher().finish() % KEY_SPAN) as u8;
    cell[5] = (RandomState::new().build_hasher().finish() % KEY_SPAN) as u8;
    cell[6] = (RandomState::new().build_hasher().finish() % KEY_SPAN) as u8;

    let fourth = (RandomState::new().build_hasher().finish() % KEY_SPAN) as u8;
    cell[7] = fourth;

    cell[0] ^= fourth;
    cell[1] ^= cell[6];

    let first = cell[4];

    cell[2] ^= cell[5];
    cell[3] ^= first;
}
