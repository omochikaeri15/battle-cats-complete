use crate::Fault;

use super::{AppContext, Base};

pub fn cannon_makes_wave(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(
        faction,
        0,
        Base::CANNON_MAKES_WAVE,
    ))? != 0)
}
