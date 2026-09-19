use crate::Fault;

use super::{AppContext, get_stages_cleared, set_stages_cleared};

pub fn add_stages_cleared(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    star: i32,
    delta: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    let value = get_stages_cleared(ctx, map_type, map_idx, star, use_cache)?.wrapping_add(delta);

    set_stages_cleared(ctx, map_type, map_idx, star, value, use_cache)
}
