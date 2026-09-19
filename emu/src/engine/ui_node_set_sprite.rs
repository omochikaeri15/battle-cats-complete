use std::rc::Rc;

use crate::Fault;

use super::{imgcut_get_cut_count, imgcut_get_sprite_cut, make_image_sprite, texture_get_height, texture_get_width, ui_node_set_color, Imgcut, Sprite, Surface};

pub fn ui_node_set_sprite(sheet: &Rc<Imgcut>, x: i32, y: i32, cut: i32) -> Result<Box<Sprite>, Fault> {
    let mut sprite = make_image_sprite();

    sprite.sheet = Some(Rc::clone(sheet));
    sprite.x = x as f32;
    sprite.y = y as f32;

    if cut == -1 {
        sprite.cut[0] = 0;
        sprite.cut[1] = 0;
        sprite.cut[2] = texture_get_width(Surface::Sheet(sheet));
        sprite.cut[3] = texture_get_height(Surface::Sheet(sheet));
    } else if imgcut_get_cut_count(sheet) as i32 > cut {
        sprite.cut = *imgcut_get_sprite_cut(sheet, cut)?;
    }

    sprite.size = [-10000, -10000];
    ui_node_set_color(&mut sprite, 0xff, 0xff, 0xff);

    Ok(sprite)
}
