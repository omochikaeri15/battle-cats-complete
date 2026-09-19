use crate::{Fault, operation};

use super::{
    AppContext, draw_context, draw_model, get_base_pos_x, get_drawable_width, maanim_execute,
};

pub fn base_guard_notice_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.i32_at(AppContext::BASE_GUARD_NOTICE)? | 2 != 3 {
        return Ok(());
    }

    let pos = get_base_pos_x(ctx, 1)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
    let width = get_drawable_width(ctx)?;
    let state = ctx.i32_at(AppContext::BASE_GUARD_NOTICE)?;
    let anim = match state {
        3 => &ctx.guard_e_breaker_anim,
        1 => &ctx.guard_e_anim,
        _ => return Ok(()),
    };
    let frame = ctx.i32_at(AppContext::BASE_GUARD_NOTICE_FRAME)?;

    maanim_execute(&mut ctx.guard_e_model, Some(anim), frame, 0)?;

    let x = operation::cvttsd2si(
        width.wrapping_add(-0x3c0) as f64 * 0.5 + operation::div_10(pos) as f64,
    );

    draw_model(draw_context(&mut ctx.draw)?, &ctx.guard_e_model, x, 0x190);

    Ok(())
}
