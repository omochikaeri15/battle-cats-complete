use crate::Fault;

use super::{AppContext, Base};

pub fn set_base_occupant(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::OCCUPANT), 1)
}
