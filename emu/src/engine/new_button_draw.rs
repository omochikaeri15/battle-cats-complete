use crate::Fault;

use super::{image_sprite_draw, scale9_image_sprite_draw, ui_node_set_color, ui_node_set_offset, AppContext, Button, SpriteKind};

const SITE: &str = "new_button_draw";

fn draw_node(ctx: &mut AppContext, button: &mut Button, x: i32, y: i32) -> Result<(), Fault> {
    if button.enabled == 0 {
        return Ok(());
    }

    if let Some(node) = button.node.as_mut().filter(|_| button.tinted != 0) {
        if button.lit != 0 {
            ui_node_set_color(node, 0xff, 0xff, 0xff);
        } else {
            ui_node_set_color(node, 0x7f, 0x7f, 0x7f);
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

pub fn new_button_draw(ctx: &mut AppContext, button: i32, x: i32, y: i32) -> Result<(), Fault> {
    let mut held = ctx
        .buttons
        .buttons
        .get_mut(&button)
        .and_then(|slot| slot.take())
        .ok_or(Fault::NullPointer { site: SITE })?;
    let drawn = draw_node(ctx, &mut held, x, y);

    if let Some(slot) = ctx.buttons.buttons.get_mut(&button) {
        *slot = Some(held);
    }

    drawn
}
