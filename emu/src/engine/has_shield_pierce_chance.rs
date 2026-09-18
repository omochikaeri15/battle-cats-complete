use crate::Fault;

use super::AppContext;

pub fn has_shield_pierce_chance(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83bd4))? != 0)
}
