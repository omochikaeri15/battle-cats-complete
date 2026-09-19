use crate::Fault;

use super::{call_rng, collect_lose_tip_candidates, AppContext};

const SITE: &str = "pick_lose_tip";

pub fn pick_lose_tip(ctx: &mut AppContext) -> Result<i32, Fault> {
    let candidates = collect_lose_tip_candidates(ctx)?;
    let count = candidates.len() as i32;

    if count <= 0 {
        return Ok(0);
    }

    let pick = call_rng(ctx, count);

    candidates.get(pick as i64 as usize).copied().ok_or(Fault::IndexOutOfRange { site: SITE, index: pick as i64, limit: count as i64 })
}
