use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn set_revive_time(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    value: i32,
) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return Ok(());
    }

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::REVIVE_TIME),
        value,
    )
}
