use crate::Fault;

use super::{get_stage_unlock, set_stage_unlock, AppContext};

pub fn add_stage_unlock(ctx: &mut AppContext, map_type: i32, map_idx: i32, star: i32, delta: i32, use_cache: i32) -> Result<(), Fault> {
    let value = get_stage_unlock(ctx, map_type, map_idx, star, use_cache)?.wrapping_add(delta);

    set_stage_unlock(ctx, map_type, map_idx, star, value, use_cache)
}
