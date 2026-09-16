use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

const SEED_SPAN: u64 = 0x7fff_ffff;

pub fn rng(seed: &mut u32, bound: i32) -> i32 {
    if bound <= 0 {
        return 0;
    }

    while *seed == 0 {
        let drawn = RandomState::new().build_hasher().finish();
        *seed = (drawn % SEED_SPAN) as u32 + 1;
    }

    let mut x = *seed;

    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 15;

    *seed = x;

    (x % bound as u32) as i32
}
