use crate::Fault;

use super::AppContext;

pub fn stage_not_sealed(ctx: &AppContext, enemy_id: i32) -> Result<bool, Fault> {
    if !ctx.altar_unsealed.contains_key(&enemy_id) {
        return Ok(true);
    }

    Ok(*ctx
        .altar_unsealed
        .get(&enemy_id)
        .ok_or(Fault::key_not_found(enemy_id as i64))?)
}
