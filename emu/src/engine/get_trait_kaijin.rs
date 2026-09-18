use crate::Fault;

use super::AppContext;

pub fn get_trait_kaijin(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83cc8))? != 0)
}
