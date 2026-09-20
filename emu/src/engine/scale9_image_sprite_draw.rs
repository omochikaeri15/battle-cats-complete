use crate::{Fault, ops};

use super::{
    AppContext, Sprite, SpriteKind, draw_context, draw_panel, image_sprite_draw,
    scale9_image_sprite_update, set_color, set_tint,
};

pub fn scale9_image_sprite_draw(
    ctx: &mut AppContext,
    sprite: &mut Sprite,
    parent: Option<([f32; 6], [f32; 2])>,
) -> Result<(), Fault> {
    scale9_image_sprite_update(sprite, parent);

    if sprite.visible != 0 {
        let sheet = sprite.sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let labelled = sheet.whole != 0;
        let saved = if labelled {
            draw_context(&mut ctx.draw)?.tint()
        } else {
            draw_context(&mut ctx.draw)?.color()
        };

        if labelled {
            set_tint(
                draw_context(&mut ctx.draw)?,
                sprite.color[0],
                sprite.color[1],
                sprite.color[2],
                sprite.alpha,
            );
        } else {
            set_color(
                draw_context(&mut ctx.draw)?,
                sprite.color[0],
                sprite.color[1],
                sprite.color[2],
                sprite.alpha,
            );
        }

        let x = ops::cvttss2si(sprite.quad[0]);
        let y = ops::cvttss2si(sprite.quad[1]);
        let width = ops::cvttss2si(sprite.quad[4] - sprite.quad[2]);
        let height = ops::cvttss2si(sprite.quad[3] - sprite.quad[1]);

        draw_panel(
            draw_context(&mut ctx.draw)?,
            sheet,
            x,
            y,
            width,
            height,
            sprite.border_scale,
            sprite.border[0],
            sprite.border[1],
        );

        if labelled {
            set_tint(
                draw_context(&mut ctx.draw)?,
                saved[0],
                saved[1],
                saved[2],
                saved[3],
            );
        } else {
            set_color(
                draw_context(&mut ctx.draw)?,
                saved[0],
                saved[1],
                saved[2],
                saved[3],
            );
        }
    }

    let handed = (sprite.transform, sprite.world_scale);

    for child in sprite.children.iter_mut() {
        match child.kind {
            SpriteKind::Image => image_sprite_draw(ctx, child, Some(handed))?,
            SpriteKind::Scale9 => scale9_image_sprite_draw(ctx, child, Some(handed))?,
        }
    }

    Ok(())
}
