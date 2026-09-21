use crate::{
    Fault,
    engine::{AppContext, Pinch},
};

pub fn queue_touch_position(ctx: &mut AppContext, x: i32, y: i32) -> Result<(), Fault> {
    let tablet = ctx.platform().ok_or(Fault::host_missing())?.is_tablet();
    let shift = if tablet { ctx.i32_at(AppContext::LETTERBOX_SHIFT)? } else { 0 };
    let lifted = shift.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_PAD)?);

    ctx.set_i32_at(AppContext::TOUCH_PENDING_X, x)?;
    ctx.set_i32_at(AppContext::TOUCH_PENDING_Y, y.wrapping_sub(lifted))
}

pub fn queue_touch_press(ctx: &mut AppContext, x: i32, y: i32) -> Result<(), Fault> {
    queue_touch_position(ctx, x, y)?;
    ctx.set_block_at::<1>(AppContext::TOUCH_PENDING_BEGAN, [1])
}

pub fn queue_touch_release(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::TOUCH_PENDING_RELEASED, [1])
}

pub fn pump_touch(ctx: &mut AppContext) -> Result<(), Fault> {
    let x = ctx.i32_at(AppContext::TOUCH_PENDING_X)?;
    let y = ctx.i32_at(AppContext::TOUCH_PENDING_Y)?;

    let held_x = ctx.i32_at(AppContext::TOUCH_X)?;
    let held_y = ctx.i32_at(AppContext::TOUCH_Y)?;

    ctx.set_i32_at(AppContext::TOUCH_PREV_X, held_x)?;
    ctx.set_i32_at(AppContext::TOUCH_PREV_Y, held_y)?;
    ctx.set_i32_at(AppContext::TOUCH_X, x)?;
    ctx.set_i32_at(AppContext::TOUCH_Y, y)?;

    if ctx.u8_at(AppContext::TOUCH_PENDING_RELEASED)? != 0 {
        ctx.set_block_at::<2>(AppContext::TOUCH_PENDING_RELEASED, [0; 2])?;
        ctx.set_block_at::<4>(AppContext::TOUCH_BEGAN, [0, 0, 0, 1])?;

        return Ok(());
    }

    if ctx.u8_at(AppContext::TOUCH_PENDING_BEGAN)? != 0 {
        ctx.set_block_at::<1>(AppContext::TOUCH_DOWN_LATCH, [1])?;
        ctx.set_block_at::<4>(AppContext::TOUCH_BEGAN, [1, 0, 1, 0])?;
        ctx.set_i32_at(AppContext::TOUCH_PREV_X, x)?;
        ctx.set_i32_at(AppContext::TOUCH_START_X, x)?;
        ctx.set_i32_at(AppContext::TOUCH_PREV_Y, y)?;
        ctx.set_i32_at(AppContext::TOUCH_START_Y, y)?;

        return Ok(());
    }

    ctx.set_block_at::<1>(AppContext::TOUCH_RELEASED, [0])?;
    ctx.set_block_at::<1>(AppContext::TOUCH_BEGAN, [0])
}

pub fn pump_pinch(ctx: &mut AppContext, gap: Option<i32>) -> Result<(), Fault> {
    let pinch = AppContext::PINCH;
    let down = gap.is_some() as u8;

    ctx.set_block_at::<1>(pinch + Pinch::FIRST_DOWN, [down])?;
    ctx.set_block_at::<1>(pinch + Pinch::SECOND_DOWN, [down])?;
    ctx.set_i32_at(pinch + Pinch::FIRST_X, 0)?;
    ctx.set_i32_at(pinch + Pinch::FIRST_Y, 0)?;
    ctx.set_i32_at(pinch + Pinch::SECOND_X, gap.unwrap_or(0))?;
    ctx.set_i32_at(pinch + Pinch::SECOND_Y, 0)
}
