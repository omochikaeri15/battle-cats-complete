use super::{AppContext, rng};

pub fn call_rng(ctx: &mut AppContext, bound: i32) -> i32 {
    let mut seed = ctx.rng_state();
    let drawn = rng(&mut seed, bound);
    ctx.set_rng_state(seed);

    drawn
}
