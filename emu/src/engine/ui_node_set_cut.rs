use crate::Fault;

use super::{imgcut_get_cut_count, imgcut_get_sprite_cut, texture_get_height, texture_get_width, Sprite, Surface};

const SITE: &str = "ui_node_set_cut";

pub fn ui_node_set_cut(sprite: &mut Sprite, cut: i32) -> Result<(), Fault> {
    let sheet = sprite.sheet.clone().ok_or(Fault::NullPointer { site: SITE })?;

    if cut == -1 {
        sprite.cut[0] = 0;
        sprite.cut[1] = 0;
        sprite.cut[2] = texture_get_width(Surface::Sheet(&sheet));
        sprite.cut[3] = texture_get_height(Surface::Sheet(&sheet));
    } else if imgcut_get_cut_count(&sheet) as i32 > cut {
        sprite.cut = *imgcut_get_sprite_cut(&sheet, cut)?;
    }

    Ok(())
}
