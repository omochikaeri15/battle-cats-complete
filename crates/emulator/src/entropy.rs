use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

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
            let seeded = counter.get().unwrap_or_else(|| {
                let clock = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_nanos() as u64);
                let place = counter as *const Cell<Option<u64>> as usize as u64;

                clock ^ place.rotate_left(32)
            });
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
