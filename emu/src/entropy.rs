use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

thread_local! {
    static COUNTER: Cell<Option<u64>> = const { Cell::new(None) };
}

pub struct Entropy;

impl Entropy {
    pub fn draw() -> u64 {
        COUNTER.with(|counter| {
            let seeded = counter
                .get()
                .unwrap_or_else(|| RandomState::new().build_hasher().finish());
            let state = seeded.wrapping_add(0x9e37_79b9_7f4a_7c15);

            counter.set(Some(state));

            let mut mixed = state;

            mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);

            mixed ^ (mixed >> 31)
        })
    }
}
