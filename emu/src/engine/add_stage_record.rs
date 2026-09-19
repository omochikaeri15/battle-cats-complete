use crate::Fault;

use super::{AppContext, get_stage_record, set_stage_record};

pub fn add_stage_record(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    stage: i32,
    star: i32,
    delta: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    let total =
        get_stage_record(ctx, map_type, map_idx, stage, star, use_cache)?.wrapping_add(delta);

    set_stage_record(ctx, map_type, map_idx, stage, star, total, use_cache)
}
