use crate::Fault;

use super::ButtonBank;

const SITE: &str = "new_button_get_y";

pub fn new_button_get_y(bank: &ButtonBank, button: i32) -> Result<i32, Fault> {
    Ok(bank
        .buttons
        .get(&button)
        .and_then(|slot| slot.as_deref())
        .ok_or(Fault::NullPointer { site: SITE })?
        .y)
}
