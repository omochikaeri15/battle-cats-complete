use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

thread_local! {
    static COUNTER: Cell<Option<u64>> = const { Cell::new(None) };
}

fn mix(state: u64) -> u64 {
    let mut mixed = state;

    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);

    mixed ^ (mixed >> 31)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entropy {
    seed: u64,
    state: u64,
}

impl Entropy {
    pub fn draw() -> u64 {
        COUNTER.with(|counter| {
            let seeded = counter
                .get()
                .unwrap_or_else(|| RandomState::new().build_hasher().finish());
            let state = seeded.wrapping_add(GAMMA);

            counter.set(Some(state));

            mix(state)
        })
    }

    pub fn loose() -> Self {
        Self::seeded(Self::draw())
    }

    pub fn seeded(seed: u64) -> Self {
        Self { seed, state: seed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn roll(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GAMMA);

        mix(self.state)
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Self::loose()
    }
}
