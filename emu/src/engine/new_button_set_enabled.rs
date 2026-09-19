use crate::Fault;

use super::ButtonBank;

const SITE: &str = "new_button_set_enabled";

pub fn new_button_set_enabled(bank: &mut ButtonBank, button: i32, enabled: u8) -> Result<i32, Fault> {
    bank.buttons.get_mut(&button).and_then(|slot| slot.as_deref_mut()).ok_or(Fault::NullPointer { site: SITE })?.enabled = enabled;

    Ok(button)
}
