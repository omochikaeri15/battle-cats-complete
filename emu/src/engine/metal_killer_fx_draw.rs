use crate::{operation, Fault};

use super::{draw_context, draw_model, get_drawable_width, maanim_execute, AppContext};

pub fn metal_killer_fx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.metal_killer_fx.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        maanim_execute(&mut ctx.metal_strong_model, Some(&ctx.metal_killer_fx_anim), ctx.metal_killer_fx[index].frame, 0)?;

        let pos_x = operation::div_10(ctx.metal_killer_fx[index].pos_x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pos_x).wrapping_add(0x44);
        let y = operation::div_10(ctx.metal_killer_fx[index].pos_y).wrapping_add(0x4d);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.metal_strong_model, x, y);
        index += 1;

        if index >= ctx.metal_killer_fx.len() {
            break;
        }
    }

    Ok(())
}
