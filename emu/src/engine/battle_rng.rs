use super::{rng_core, AppContext};

pub fn battle_rng(ctx: &mut AppContext, bound: i32) -> i32 {
    let mut seed = ctx.rng_state();
    let drawn = rng_core(&mut seed, bound);
    ctx.set_rng_state(seed);

    drawn
}
