use crate::{Fault, ops};

use super::{AppContext, draw_context, draw_model, get_drawable_width, maanim_execute};

pub fn savage_vfx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.savage_vfx.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        maanim_execute(
            &mut ctx.strong_attack_model,
            Some(&ctx.savage_vfx_anim),
            ctx.savage_vfx[index].frame,
            0,
        )?;

        let pos_x = ops::div_10(
            ctx.savage_vfx[index]
                .pos_x
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
            .wrapping_add(pos_x)
            .wrapping_add(0x44);
        let y = ops::div_10(ctx.savage_vfx[index].pos_y).wrapping_add(0x4d);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.strong_attack_model, x, y);
        index += 1;

        if index >= ctx.savage_vfx.len() {
            break;
        }
    }

    Ok(())
}
