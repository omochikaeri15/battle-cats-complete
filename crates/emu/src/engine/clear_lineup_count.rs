use crate::Fault;

use super::{AppContext, get_global_map_id, get_stage_index, get_crown_level};

pub fn clear_lineup_count(
    ctx: &mut AppContext,
    map: i32,
    stage: i32,
    crown: i32,
) -> Result<i32, Fault> {
    let (map, stage, crown) = if map == -1 {
        (
            get_global_map_id(ctx, 0)?,
            get_stage_index(ctx)?,
            get_crown_level(ctx)?,
        )
    } else {
        (map, stage, crown)
    };
    let key = map
        .wrapping_mul(0x3e8)
        .wrapping_add(stage.wrapping_mul(10))
        .wrapping_add(crown);

    if !ctx.clear_lineups.contains_key(&key) {
        return Ok(0);
    }

    Ok(ctx.clear_lineups.entry(key).or_default().len() as i32)
}
