use crate::Fault;

use super::{get_global_map_id, get_stage_index, get_star_level, AppContext};

pub fn clear_lineup_count(ctx: &mut AppContext, map: i32, stage: i32, star: i32) -> Result<i32, Fault> {
    let (map, stage, star) = if map == -1 { (get_global_map_id(ctx, 0)?, get_stage_index(ctx)?, get_star_level(ctx)?) } else { (map, stage, star) };
    let key = map.wrapping_mul(0x3e8).wrapping_add(stage.wrapping_mul(10)).wrapping_add(star);

    if !ctx.clear_lineups.contains_key(&key) {
        return Ok(0);
    }

    Ok(ctx.clear_lineups.entry(key).or_default().len() as i32)
}
