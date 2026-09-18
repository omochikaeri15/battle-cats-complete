use crate::fault::Fault;

use super::AppContext;

pub fn set_slot_kind(ctx: &mut AppContext, team: i32, slot: i32, mode: i32, unit_id: i32) -> Result<(), Fault> {
    let kind = match mode {
        0 => mode,
        2 => unit_id.wrapping_add(2),
        1 => 1,
        _ => return Ok(()),
    };

    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x838f8), kind)
}
