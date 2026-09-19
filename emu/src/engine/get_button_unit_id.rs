use crate::Fault;

use super::{AppContext, get_button_unit_row};

pub fn get_button_unit_id(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    Ok(get_button_unit_row(ctx, faction, slot)?.wrapping_add(-2))
}
