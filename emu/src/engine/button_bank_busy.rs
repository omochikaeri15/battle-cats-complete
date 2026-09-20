use crate::Fault;

use super::ButtonBank;

pub fn button_bank_busy(bank: &ButtonBank) -> Result<bool, Fault> {
    for slot in bank.buttons.values() {
        let button = slot.as_deref().ok_or(Fault::null_pointer())?;

        if button.enabled != 0 && button.state == 1 {
            return Ok(true);
        }
    }

    Ok(false)
}
