use crate::Fault;

use super::{AppContext, get_stage_unlock, set_stage_unlock};

pub fn add_stage_unlock(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    crown: i32,
    delta: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    let value = get_stage_unlock(ctx, map_type, map_idx, crown, use_cache)?.wrapping_add(delta);

    set_stage_unlock(ctx, map_type, map_idx, crown, value, use_cache)
}
