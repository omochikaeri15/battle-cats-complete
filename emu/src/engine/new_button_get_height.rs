use crate::Fault;

use super::ButtonBank;

pub fn new_button_get_height(bank: &ButtonBank, button: i32) -> Result<i32, Fault> {
    Ok(bank
        .buttons
        .get(&button)
        .and_then(|slot| slot.as_deref())
        .ok_or(Fault::null_pointer())?
        .height)
}
