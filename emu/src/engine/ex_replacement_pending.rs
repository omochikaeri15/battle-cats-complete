use crate::Fault;

use super::{get_global_map_id, get_stage_index, AppContext};

pub fn ex_replacement_pending(ctx: &mut AppContext, mut map_id: i32, mut stage: i32) -> Result<bool, Fault> {
    if map_id == -1 || stage != -1 {
        if map_id == -1 {
            map_id = get_global_map_id(ctx, 0)?;
        }

        if stage == -1 {
            stage = get_stage_index(ctx)?;
        }

        if !ctx.ex_replacement_stages.contains_key(&map_id) {
            return Ok(false);
        }

        return Ok(ctx.ex_replacement_stages.entry(map_id).or_default().contains(&stage));
    }

    if !ctx.ex_replacement_stages.contains_key(&map_id) {
        return Ok(false);
    }

    Ok(!ctx.ex_replacement_stages.entry(map_id).or_default().is_empty())
}
