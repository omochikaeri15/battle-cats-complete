use crate::Fault;

use super::{AppContext, get_global_map_id, get_stage_index, get_crown_level};

pub fn find_fixed_lineup(
    ctx: &mut AppContext,
    mut map_id: i32,
    mut stage: i32,
    mut level: i32,
) -> Result<Option<String>, Fault> {
    if map_id == -1 {
        map_id = get_global_map_id(ctx, 0)?;
        stage = get_stage_index(ctx)?;
        level = get_crown_level(ctx)?;
    }

    for row in &ctx.fixed_lineup_store.rows {
        if row.map_id == map_id && row.stage == stage && row.level == level {
            return Ok(Some(row.name.clone()));
        }
    }

    Ok(None)
}
