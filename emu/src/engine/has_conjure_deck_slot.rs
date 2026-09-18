use crate::Fault;

use super::AppContext;

pub fn has_conjure_deck_slot(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83c50))? >= 0)
}
