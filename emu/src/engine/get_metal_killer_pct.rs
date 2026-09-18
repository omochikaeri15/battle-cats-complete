use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn get_metal_killer_pct(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::entity_field(faction, slot, Entity::METAL_KILLER_PCT))
}
