use crate::Fault;

use super::ButtonBank;

pub fn new_button_set_back_key(
    bank: &mut ButtonBank,
    button: i32,
    back_key: u8,
) -> Result<i32, Fault> {
    bank.buttons
        .get_mut(&button)
        .and_then(|slot| slot.as_deref_mut())
        .ok_or(Fault::null_pointer())?
        .back_key = back_key;

    Ok(button)
}
