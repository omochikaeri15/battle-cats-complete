use crate::Fault;

use super::AppContext;

pub fn get_counter_surge(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83c48))? != 0)
}
