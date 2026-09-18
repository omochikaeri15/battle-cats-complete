use crate::Fault;

use super::AppContext;

pub fn get_attack_count(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, 0x8394c))? == 0 {
        return Ok(0);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, 0x83adc))? == 0 {
        return Ok(1);
    }

    let third_missing = ctx.i32_at(AppContext::entity_field(faction, slot, 0x83ae0))? == 0;

    Ok(3 - third_missing as i32)
}
