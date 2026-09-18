use crate::Fault;

use super::AppContext;

pub fn has_weaken_chance(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83a08))? != 0)
}
