use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn add_revive_timer(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    delta: i32,
) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(());
    }

    let field = AppContext::entity_field(faction, slot, Entity::REVIVE_TIMER);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
