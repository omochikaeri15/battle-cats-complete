use crate::Fault;

use super::{AppContext, get_global_map_id};

pub fn find_score_bonus(ctx: &mut AppContext, kind: i32) -> Result<Option<&Vec<i32>>, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    if !ctx.score_bonus_maps.contains_key(&map_id) {
        return Ok(None);
    }

    if !ctx
        .score_bonus_maps
        .entry(map_id)
        .or_default()
        .bonuses
        .contains_key(&kind)
    {
        return Ok(None);
    }

    Ok(Some(
        ctx.score_bonus_maps
            .entry(map_id)
            .or_default()
            .bonuses
            .entry(kind)
            .or_default(),
    ))
}
