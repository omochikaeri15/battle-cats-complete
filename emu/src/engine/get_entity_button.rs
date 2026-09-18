use crate::Fault;

use super::{get_button_unit_row, AppContext, Entity};

pub fn get_entity_button(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    let buttons = if faction == 0 { 0x15 } else { 0xa };
    let mut button = 0i32;

    loop {
        let row = get_button_unit_row(ctx, faction, button)?;

        if row == ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? {
            return Ok(button);
        }

        button += 1;

        if buttons == button {
            return Ok(-1);
        }
    }
}
