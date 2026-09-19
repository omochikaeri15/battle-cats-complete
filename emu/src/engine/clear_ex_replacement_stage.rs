use crate::Fault;

use super::{AppContext, map_type_base_id, validate_map_type};

pub fn clear_ex_replacement_stage(ctx: &mut AppContext) -> Result<(), Fault> {
    let key = map_type_base_id(
        validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?),
        ctx.i32_at(AppContext::MAP_INDEX)?,
    );

    if !ctx.ex_replacement_stages.contains_key(&key) {
        return Ok(());
    }

    let stage = ctx.i32_at(AppContext::STAGE_INDEX)?;
    let stages = ctx.ex_replacement_stages.entry(key).or_default();

    if let Some(at) = stages.iter().position(|value| *value == stage) {
        stages.remove(at);

        if ctx.ex_replacement_stages.entry(key).or_default().is_empty() {
            ctx.ex_replacement_stages.remove(&key);
        }
    }

    Ok(())
}
