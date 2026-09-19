use super::ButtonBank;

pub fn button_bank_find(bank: &ButtonBank, id: i32) -> Option<i32> {
    bank.buttons.get(&id).and_then(|button| button.as_ref().map(|_| id))
}
