use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn set_metal(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::METAL_CAT), value)
    } else {
        ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_METAL), value)
    }
}
