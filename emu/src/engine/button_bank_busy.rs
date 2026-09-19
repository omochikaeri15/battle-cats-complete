use crate::Fault;

use super::ButtonBank;

const SITE: &str = "button_bank_busy";

pub fn button_bank_busy(bank: &ButtonBank) -> Result<bool, Fault> {
    for slot in bank.buttons.values() {
        let button = slot.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        if button.enabled != 0 && button.state == 1 {
            return Ok(true);
        }
    }

    Ok(false)
}
