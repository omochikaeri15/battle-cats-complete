use crate::Fault;

use super::{AppContext, Base};

pub fn has_cannon_ready_fx(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, 0, Base::CANNON_READY_FX))? != 0)
}
