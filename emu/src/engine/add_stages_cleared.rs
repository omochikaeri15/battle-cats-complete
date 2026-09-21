use crate::Fault;

use super::{AppContext, get_stages_cleared, set_stages_cleared};

pub fn add_stages_cleared(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    crown: i32,
    delta: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    let value = get_stages_cleared(ctx, map_type, map_idx, crown, use_cache)?.wrapping_add(delta);

    set_stages_cleared(ctx, map_type, map_idx, crown, value, use_cache)
}
