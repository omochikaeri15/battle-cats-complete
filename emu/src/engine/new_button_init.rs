use super::{AppContext, Sprite};
use crate::Fault;

pub type ButtonHandler = fn(&mut AppContext, i32, i32) -> Result<(), Fault>;

#[derive(Clone, Default)]
pub struct Button {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub node: Option<Box<Sprite>>,
    pub handler: Option<ButtonHandler>,
    pub state: i32,
    pub touching: u8,
    pub enabled: u8,
    pub touchable: u8,
    pub pressable: u8,
    pub animated: u8,
    pub back_key: u8,
    pub tinted: u8,
    pub lit: u8,
    pub frame: i32,
    pub press_scales: Vec<f32>,
}

#[allow(clippy::too_many_arguments)]
pub fn new_button_init(button: &mut Button, id: i32, x: i32, y: i32, width: i32, height: i32, node: Option<Box<Sprite>>, handler: Option<ButtonHandler>) {
    button.id = id;
    button.x = x;
    button.y = y;
    button.offset_x = 0;
    button.offset_y = 0;
    button.width = width;
    button.height = height;
    button.node = node;
    button.handler = handler;
    button.state = 0;
    button.touching = 0;
    button.lit = 1;
    button.enabled = 1;
    button.touchable = 1;
    button.pressable = 1;
    button.animated = 1;
    button.back_key = 0;
    button.tinted = 0;
    button.press_scales = vec![1.0, 1.0, 1.1, 0.9, 1.0, 1.0];
}
