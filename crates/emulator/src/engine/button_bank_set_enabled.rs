use super::ButtonBank;

pub fn button_bank_set_enabled(bank: &mut ButtonBank, enabled: u8) {
    bank.enabled = enabled;
}
