use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn add_burrow_count(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let mut count = 0i32;

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        count = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BURROW_COUNT))?;
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::BURROW_COUNT), count.wrapping_add(delta))?;
    }

    Ok(())
}
