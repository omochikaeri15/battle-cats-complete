use crate::{Fault, ops};

use super::{AppContext, draw_context, draw_model, get_drawable_width, maanim_execute};

pub fn drain_vfx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.drain_vfx.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        maanim_execute(
            &mut ctx.recast_decrease_e_model,
            Some(&ctx.drain_vfx_anim),
            ctx.drain_vfx[index].frame,
            0,
        )?;

        let pos_x = ops::div_10(
            ctx.drain_vfx[index]
                .pos_x
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
            .wrapping_add(pos_x)
            .wrapping_add(0x44);
        let y = ops::div_10(ctx.drain_vfx[index].pos_y).wrapping_add(0x4d);

        draw_model(
            draw_context(&mut ctx.draw)?,
            &ctx.recast_decrease_e_model,
            x,
            y,
        );
        index += 1;

        if index >= ctx.drain_vfx.len() {
            break;
        }
    }

    Ok(())
}
