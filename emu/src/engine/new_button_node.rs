use crate::Fault;

use super::{ButtonBank, Sprite};

const SITE: &str = "new_button_node";

pub fn new_button_node(bank: &mut ButtonBank, button: i32) -> Result<&mut Sprite, Fault> {
    bank.buttons
        .get_mut(&button)
        .and_then(|slot| slot.as_deref_mut())
        .and_then(|button| button.node.as_deref_mut())
        .ok_or(Fault::NullPointer { site: SITE })
}
