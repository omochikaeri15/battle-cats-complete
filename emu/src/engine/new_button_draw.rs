use crate::Fault;

use super::{
    AppContext, SpriteKind, image_sprite_draw, scale9_image_sprite_draw, ui_node_set_color,
    ui_node_set_offset,
};

const SITE: &str = "new_button_draw";

pub fn new_button_draw(ctx: &mut AppContext, button: i32, x: i32, y: i32) -> Result<(), Fault> {
    let mut held = ctx
        .buttons
        .buttons
        .get_mut(&button)
        .and_then(|slot| slot.take())
        .ok_or(Fault::NullPointer { site: SITE })?;
    let mut drawn = Ok(());

    if held.enabled != 0 {
        if let Some(node) = held.node.as_mut().filter(|_| held.tinted != 0) {
            if held.lit != 0 {
                ui_node_set_color(node, 0xff, 0xff, 0xff);
            } else {
                ui_node_set_color(node, 0x7f, 0x7f, 0x7f);
            }
        }

        let across = x.wrapping_add(held.offset_x) as f32;
        let down = y.wrapping_add(held.offset_y) as f32;

        if let Some(node) = held.node.as_mut() {
            ui_node_set_offset(node, across, down);

            drawn = match node.kind {
                SpriteKind::Image => image_sprite_draw(ctx, node),
                SpriteKind::Scale9 => scale9_image_sprite_draw(ctx, node),
            };
        }
    }

    if let Some(slot) = ctx.buttons.buttons.get_mut(&button) {
        *slot = Some(held);
    }

    drawn
}
