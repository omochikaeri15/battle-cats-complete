use crate::Fault;

use super::{AppContext, Base};

pub fn set_cannon_parts(ctx: &mut AppContext, faction: i32, cannon_id: i32, foundation_id: i32, decor_id: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CANNON_ID), cannon_id)?;
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CANNON_FOUNDATION_ID), foundation_id)?;
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CANNON_DECOR_ID), decor_id)
}
