use crate::{Fault, operation};

use super::{AppContext, draw_context, draw_model, get_drawable_width, maanim_execute};

pub fn toxic_vfx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.toxic_vfx.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        maanim_execute(
            &mut ctx.percentage_attack_model,
            Some(&ctx.toxic_vfx_anim),
            ctx.toxic_vfx[index].frame,
            0,
        )?;

        let pos_x = operation::div_10(
            ctx.toxic_vfx[index]
                .pos_x
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
            .wrapping_add(pos_x)
            .wrapping_add(0x44);
        let y = operation::div_10(ctx.toxic_vfx[index].pos_y).wrapping_add(0x4d);

        draw_model(
            draw_context(&mut ctx.draw)?,
            &ctx.percentage_attack_model,
            x,
            y,
        );
        index += 1;

        if index >= ctx.toxic_vfx.len() {
            break;
        }
    }

    Ok(())
}
