use crate::Fault;

use super::ButtonBank;

pub fn new_button_set_pos(bank: &mut ButtonBank, button: i32, x: i32, y: i32) -> Result<i32, Fault> {
    let this = bank
        .buttons
        .get_mut(&button)
        .and_then(|slot| slot.as_deref_mut())
        .ok_or(Fault::null_pointer())?;

    this.x = x;
    this.y = y;

    Ok(button)
}
