use crate::Fault;

use super::{AppContext, get_left_inset_logical, has_insets, touch_table_update, window_to_design};

pub fn app_on_touch(ctx: &mut AppContext, kind: i32, x: i32, y: i32, id: i32) -> Result<(), Fault> {
    let mut x = x;
    let mut y = y;

    window_to_design(ctx, &mut x, &mut y)?;

    let mut shift = 0i32;

    if ctx.platform().ok_or(Fault::host_missing())?.is_tablet() {
        shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    }

    y = y.wrapping_sub(shift.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_PAD)?));

    if has_insets(ctx) && ctx.i32_at(AppContext::SCENE_ID)? == 0x64 {
        let page = ctx.i32_at(AppContext::SCENE_0X64_PAGE)?;

        if page != 0 && page != 0x1869f && ctx.u8_at(AppContext::INSETS_IGNORED)? == 0 {
            x = x.wrapping_sub(get_left_inset_logical(ctx));
        }
    }

    touch_table_update(ctx, kind, x, y, id)?;

    match kind {
        2 => {
            if ctx.i32_at(AppContext::TOUCH_ID)? == id {
                ctx.set_block_at::<1>(AppContext::TOUCH_PENDING_RELEASED, [1])?;
                ctx.set_i32_at(AppContext::TOUCH_PENDING_X, x)?;
                ctx.set_i32_at(AppContext::TOUCH_PENDING_Y, y)?;
            }
        }
        1 => {
            if ctx.i32_at(AppContext::TOUCH_ID)? == id {
                ctx.set_i32_at(AppContext::TOUCH_PENDING_X, x)?;
                ctx.set_i32_at(AppContext::TOUCH_PENDING_Y, y)?;
            }
        }
        0 => {
            ctx.set_i32_at(AppContext::TOUCH_ID, id)?;
            ctx.set_block_at::<1>(AppContext::TOUCH_PENDING_BEGAN, [1])?;
            ctx.set_i32_at(AppContext::TOUCH_PENDING_X, x)?;
            ctx.set_i32_at(AppContext::TOUCH_PENDING_Y, y)?;
        }
        _ => {}
    }

    Ok(())
}
