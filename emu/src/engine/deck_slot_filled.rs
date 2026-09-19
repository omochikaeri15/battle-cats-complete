use crate::Fault;

use super::{AppContext, get_button_unit_row};

pub fn deck_slot_filled(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(get_button_unit_row(ctx, faction, slot)? != -1)
}
