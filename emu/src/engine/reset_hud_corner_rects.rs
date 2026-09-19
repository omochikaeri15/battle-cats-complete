use crate::Fault;

use super::{
    AppContext, get_drawable_width, get_left_inset_logical, get_right_inset_logical,
    get_top_inset_offset,
};

pub fn reset_hud_corner_rects(ctx: &mut AppContext) -> Result<(), Fault> {
    let width = get_drawable_width(ctx)?;
    let right = get_right_inset_logical(ctx)?;

    ctx.set_i32_at(
        AppContext::CANNON_RECT,
        width.wrapping_sub(right).wrapping_add(-0x92),
    )?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    ctx.set_i32_at(
        AppContext::CANNON_RECT + 4,
        get_top_inset_offset(ctx)
            .wrapping_add(shift)
            .wrapping_add(0x1fe),
    )?;
    ctx.set_i32_at(AppContext::CANNON_RECT + 8, 0xc2)?;
    ctx.set_i32_at(AppContext::CANNON_RECT + 0xc, 0x82)?;
    ctx.set_i32_at(
        AppContext::WORKER_RECT,
        get_left_inset_logical(ctx).wrapping_add(-0x30),
    )?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    ctx.set_i32_at(
        AppContext::WORKER_RECT + 4,
        get_top_inset_offset(ctx)
            .wrapping_add(shift)
            .wrapping_add(0x207),
    )?;
    ctx.set_i32_at(AppContext::WORKER_RECT + 8, 0xc2)?;
    ctx.set_i32_at(AppContext::WORKER_RECT + 0xc, 0x7d)
}
