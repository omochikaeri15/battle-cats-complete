use crate::Fault;

use super::AppContext;

pub fn get_shield_fx_frame(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(faction, slot, 0x83bfc))
}
