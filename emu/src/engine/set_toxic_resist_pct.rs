use crate::Fault;

use super::AppContext;

pub fn set_toxic_resist_pct(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83bc8), value)
}
