use crate::Fault;

use super::{compute_back_bound, AppContext, Entity};

pub fn keep_in_bound(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    compute_back_bound(ctx, faction, slot)?;

    if faction == 1 {
        if ctx.i32_at(AppContext::entity_field(1, slot, Entity::BACK_BOUND))? < ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))? {
            return Ok(());
        }
    } else {
        if faction != 0 {
            return Ok(());
        }

        if ctx.i32_at(AppContext::entity_field(0, slot, Entity::BACK_BOUND))? > ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))? {
            return Ok(());
        }
    }

    let bound = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BACK_BOUND))?;

    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::POS_X), bound)
}
