use crate::{Entropy, engine::AppContext};

const SEED_SPAN: u64 = 0x7fff_ffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seeds {
    pub rng: u32,
    pub entropy: u64,
}

impl Seeds {
    pub fn draw() -> Self {
        Self {
            rng: (Entropy::draw() % SEED_SPAN) as u32 + 1,
            entropy: Entropy::draw(),
        }
    }
}

pub fn plant_seeds(ctx: &mut AppContext, seeds: Seeds) {
    ctx.set_rng_state(seeds.rng);
    ctx.entropy = Entropy::seeded(seeds.entropy);
}
