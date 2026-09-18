use crate::Fault;

use super::{AppContext, Entity};

pub fn set_occupant(ctx: &mut AppContext, faction: i32, slot: i32, mode: i32, unit_id: i32) -> Result<(), Fault> {
    let occupant = match mode {
        0 => mode,
        2 => unit_id.wrapping_add(2),
        1 => 1,
        _ => return Ok(()),
    };

    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT), occupant)
}
