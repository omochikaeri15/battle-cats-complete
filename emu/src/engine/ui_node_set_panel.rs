use std::rc::Rc;

use super::{Imgcut, Sprite, make_scale9_image_sprite, ui_node_set_color};

#[allow(clippy::too_many_arguments)]
pub fn ui_node_set_panel(
    sheet: &Rc<Imgcut>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    border_x: i32,
    border_y: i32,
    border_scale: f32,
) -> Box<Sprite> {
    let mut sprite = make_scale9_image_sprite();

    sprite.sheet = Some(Rc::clone(sheet));
    sprite.x = x as f32;
    sprite.y = y as f32;
    sprite.size = [width, height];
    ui_node_set_color(&mut sprite, 0xff, 0xff, 0xff);
    sprite.border = [border_x, border_y];
    sprite.border_scale = border_scale;

    sprite
}
