use std::collections::BTreeMap;

use super::{Button, ButtonHandler, Sprite, new_button_init};

#[derive(Clone, Default)]
pub struct ButtonBank {
    pub buttons: BTreeMap<i32, Option<Box<Button>>>,
    pub enabled: u8,
}

pub fn new_button_register(
    bank: &mut ButtonBank,
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    node: Option<Box<Sprite>>,
    handler: Option<ButtonHandler>,
) -> i32 {
    let slot = bank.buttons.entry(id).or_insert(None);

    *slot = Some(Box::default());

    if let Some(button) = slot {
        new_button_init(button, id, x, y, width, height, node, handler);
    }

    id
}
