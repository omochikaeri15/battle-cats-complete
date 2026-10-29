use crate::Fault;

use super::{AppContext, Entity, compute_back_bound};

pub fn can_push_back(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    compute_back_bound(ctx, faction, slot)?;

    if faction == 1 {
        return Ok(
            ctx.i32_at(AppContext::entity_field(1, slot, Entity::BACK_BOUND))?
                < ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?,
        );
    }

    if faction != 0 {
        return Ok(false);
    }

    Ok(
        ctx.i32_at(AppContext::entity_field(0, slot, Entity::BACK_BOUND))?
            > ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?,
    )
}
