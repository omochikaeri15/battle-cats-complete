use crate::Fault;

use super::{AppContext, Base};

pub fn set_cannon_makes_wave(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CANNON_MAKES_WAVE), value)
}
