use crate::Fault;

use super::{AppContext, Entity};

pub fn get_attack_count(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::ATTACK_1_FORESWING))? == 0 {
        return Ok(0);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::ATTACK_2_FORESWING))? == 0 {
        return Ok(1);
    }

    let third_missing = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::ATTACK_3_FORESWING))? == 0;

    Ok(3 - third_missing as i32)
}
