use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn set_trait_red(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    value: i32,
) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        ctx.set_i32_at(
            AppContext::entity_field(faction, slot, Entity::TARGETS_RED),
            value,
        )
    } else {
        ctx.set_i32_at(
            AppContext::entity_field(faction, slot, Entity::IS_RED),
            value,
        )
    }
}
