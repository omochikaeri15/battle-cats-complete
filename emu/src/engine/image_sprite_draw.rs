use crate::{Fault, operation};

use super::{
    AppContext, Sprite, SpriteKind, draw_context, draw_quad_region, image_sprite_update,
    scale9_image_sprite_draw, set_color, set_tint,
};

const SITE: &str = "image_sprite_draw";

pub fn image_sprite_draw(ctx: &mut AppContext, sprite: &mut Sprite) -> Result<(), Fault> {
    image_sprite_update(sprite, None);

    if sprite.visible != 0 {
        let sheet = sprite.sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
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

        draw_quad_region(
            draw_context(&mut ctx.draw)?,
            sheet,
            operation::cvttss2si(sprite.quad[0]),
            operation::cvttss2si(sprite.quad[1]),
            operation::cvttss2si(sprite.quad[2]),
            operation::cvttss2si(sprite.quad[3]),
            operation::cvttss2si(sprite.quad[4]),
            operation::cvttss2si(sprite.quad[5]),
            operation::cvttss2si(sprite.quad[6]),
            operation::cvttss2si(sprite.quad[7]),
            sprite.cut[0],
            sprite.cut[1],
            sprite.cut[2],
            sprite.cut[3],
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

    for child in sprite.children.iter_mut() {
        match child.kind {
            SpriteKind::Image => image_sprite_draw(ctx, child)?,
            SpriteKind::Scale9 => scale9_image_sprite_draw(ctx, child)?,
        }
    }

    Ok(())
}
