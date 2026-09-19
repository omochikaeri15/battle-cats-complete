use crate::Fault;

use super::{
    AppContext, Button, SpriteKind, image_sprite_draw, scale9_image_sprite_draw, ui_node_set_color,
    ui_node_set_offset,
};

pub fn new_button_draw(
    ctx: &mut AppContext,
    button: &mut Button,
    x: i32,
    y: i32,
) -> Result<(), Fault> {
    if button.enabled == 0 {
        return Ok(());
    }

    if button.tinted != 0 {
        if let Some(node) = button.node.as_mut() {
            if button.lit != 0 {
                ui_node_set_color(node, 0xff, 0xff, 0xff);
            } else {
                ui_node_set_color(node, 0x7f, 0x7f, 0x7f);
            }
        }
    }

    let across = x.wrapping_add(button.offset_x) as f32;
    let down = y.wrapping_add(button.offset_y) as f32;

    match button.node.as_mut() {
        Some(node) => {
            ui_node_set_offset(node, across, down);

            match node.kind {
                SpriteKind::Image => image_sprite_draw(ctx, node),
                SpriteKind::Scale9 => scale9_image_sprite_draw(ctx, node),
            }
        }
        None => Ok(()),
    }
}
