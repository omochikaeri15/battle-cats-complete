use crate::fault::Fault;

use super::AppContext;

pub fn slot_occupied(ctx: &AppContext, team: i32, slot: i32) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::entity_field(team, slot, 0x838f8))? == 0 {
        return Ok(0);
    }

    let is_unit = ctx.i32_at(AppContext::entity_field(team, slot, 0x838f8))? != 1;

    Ok(is_unit as i32 + 1)
}
