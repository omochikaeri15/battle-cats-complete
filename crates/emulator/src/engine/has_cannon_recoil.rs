use crate::Fault;

use super::{AppContext, Base};

pub fn has_cannon_recoil(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, 0, Base::CANNON_RECOIL))? != 0)
}
