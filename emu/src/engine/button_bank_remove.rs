use super::ButtonBank;

pub fn button_bank_remove(bank: &mut ButtonBank, id: i32) {
    bank.buttons.remove(&id);
}
