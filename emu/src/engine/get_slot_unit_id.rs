use crate::fault::Fault;

use super::AppContext;

pub fn get_slot_unit_id(ctx: &AppContext, team: i32, slot: i32) -> Result<i32, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(team, slot, 0x838f8))?.wrapping_sub(2))
}
