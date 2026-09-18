use crate::Fault;

use super::AppContext;

pub fn set_metal_killer_fx(ctx: &mut AppContext, team: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83c64), value)
}
