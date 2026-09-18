use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn is_metal(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    let cat_side = read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0;
    let field = if cat_side { Entity::METAL_CAT } else { Entity::TRAIT_METAL };

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, field))? != 0)
}
