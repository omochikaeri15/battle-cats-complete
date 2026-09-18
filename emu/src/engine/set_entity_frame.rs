use crate::Fault;

use super::AppContext;

pub fn set_entity_frame(ctx: &mut AppContext, faction: i32, slot: i32, frame: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83900), frame)
}
