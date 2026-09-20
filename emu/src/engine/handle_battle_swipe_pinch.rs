use crate::{Fault, ops};

use super::{
    AppContext, atan2_deg, get_max_zoom, get_pinch_delta, get_touch_prev_x, get_touch_start_x,
    get_touch_start_y, get_touch_x, get_touch_y, pinch_is_active, touch_began, touch_is_down,
    touch_released,
};

pub fn handle_battle_swipe_pinch(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::CAMERA_DRAGGING)? == 0 {
        if pinch_is_active(ctx, AppContext::PINCH)? != 0 {
            ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

            let old_zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)?;
            let old_percent = ops::div_100(old_zoom as i64) as i32;

            ctx.set_i32_at(AppContext::DRAW_TEMP_0, old_percent)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_1, old_percent)?;

            let delta = get_pinch_delta(ctx, AppContext::PINCH)?;
            let zoom = delta
                .wrapping_mul(5)
                .wrapping_shl(2)
                .wrapping_add(ctx.i32_at(AppContext::CAMERA_ZOOM)?);

            ctx.set_i32_at(AppContext::CAMERA_ZOOM, zoom)?;

            let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?.wrapping_mul(0x64);

            let new_zoom = if zoom < min_zoom {
                ctx.set_i32_at(AppContext::CAMERA_ZOOM, min_zoom)?;
                min_zoom
            } else if zoom <= get_max_zoom(ctx)? {
                ctx.i32_at(AppContext::CAMERA_ZOOM)?
            } else {
                let max_zoom = get_max_zoom(ctx)?;

                ctx.set_i32_at(AppContext::CAMERA_ZOOM, max_zoom)?;
                max_zoom
            };

            let new_percent = ops::div_100(new_zoom as i64) as i32;

            ctx.set_i32_at(AppContext::DRAW_TEMP_0, new_percent)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_2, new_percent)?;

            let old_span =
                ops::idiv(0x5b8d800, old_zoom).ok_or(Fault::divide(old_zoom as i64))?;
            let camera_x =
                ops::div_2(old_span).wrapping_add(ctx.i32_at(AppContext::CAMERA_X)?);
            let new_span =
                ops::idiv(0x5b8d800, new_zoom).ok_or(Fault::divide(new_zoom as i64))?;

            ctx.set_i32_at(
                AppContext::CAMERA_X,
                camera_x.wrapping_sub(ops::div_2(new_span)),
            )?;
            ctx.set_block_at::<1>(AppContext::PINCH_ZOOMED, [1])?;
        } else if touch_is_down(ctx)? == 0 && touch_released(ctx)? == 0 && touch_began(ctx)? == 0 {
            ctx.set_block_at::<1>(AppContext::PINCH_ZOOMED, [0])?;
        }
    }

    'inertia: {
        if touch_is_down(ctx)? == 0 {
            ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [0])?;
            ctx.set_block_at::<1>(AppContext::CAMERA_DRAGGING, [0])?;
            ctx.set_block_at::<1>(AppContext::DRAG_LATCHED, [0])?;

            break 'inertia;
        }

        if ctx.u8_at(AppContext::DECK_SWIPE_LATCHED)? != 0 {
            break 'inertia;
        }

        if ctx.u8_at(AppContext::PINCH_ZOOMED)? != 0 {
            break 'inertia;
        }

        if ctx.u8_at(AppContext::DECK_BUTTON_HELD)? != 0 {
            break 'inertia;
        }

        ctx.set_i32_at(
            AppContext::SWIPE_DY,
            get_touch_y(ctx)?.wrapping_sub(get_touch_start_y(ctx)?),
        )?;

        let rise = get_touch_y(ctx)?.wrapping_sub(get_touch_start_y(ctx)?) as f32;
        let run = get_touch_x(ctx)?.wrapping_sub(get_touch_start_x(ctx)?) as f32;
        let angle = ops::cvttss2si(atan2_deg(rise, run));

        ctx.set_i32_at(AppContext::SWIPE_ANGLE, angle)?;

        'pan: {
            let back_row = ctx.u8_at(AppContext::DECK_BACK_ROW_ENABLED)?;
            let dragging = ctx.u8_at(AppContext::CAMERA_DRAGGING)?;
            let swapping;

            if back_row == 0 || ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 || dragging != 0 {
                if dragging != 0 {
                    break 'pan;
                }

                swapping = ctx.u8_at(AppContext::DECK_ROW_SWAPPING)?;
            } else {
                let dy = ctx.i32_at(AppContext::SWIPE_DY)?;

                if dy <= -0x19 && (0xe1..=0x13b).contains(&angle) {
                    ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

                    if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
                        ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                        ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 0)?;
                    } else {
                        match ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)? {
                            -1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 => {
                                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                            }
                            1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 => {
                                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?;
                            }
                            _ => {}
                        }
                    }

                    ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
                    ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [1])?;
                    swapping = 1;
                } else {
                    let was_swapping = ctx.u8_at(AppContext::DECK_ROW_SWAPPING)?;

                    if dy >= 0x19 && angle.wrapping_sub(0x2d) as u32 <= 0x5a {
                        if was_swapping == 0 {
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 1)?;
                        } else {
                            match ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)? {
                                -1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 => {
                                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                                }
                                1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 => {
                                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?;
                                }
                                _ => {}
                            }
                        }

                        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
                        ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
                        ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [1])?;
                        swapping = 1;
                    } else {
                        swapping = was_swapping;
                    }
                }
            }

            if swapping != 0 && ctx.u8_at(AppContext::CPU_ENABLED)? == 0 {
                break 'inertia;
            }

            if (angle.wrapping_sub(0x87) as u32) < 0x5b {
                break 'pan;
            }

            if angle.wrapping_add(-0x13b) as u32 > 0xfffffef2 {
                break 'inertia;
            }
        }

        let mut drag = get_touch_x(ctx)?.wrapping_sub(get_touch_prev_x(ctx)?) < -9;

        if !drag {
            drag = get_touch_x(ctx)?.wrapping_sub(get_touch_prev_x(ctx)?) > 9
                || ctx.u8_at(AppContext::DRAG_LATCHED)? != 0;
        }

        if !drag {
            ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

            break 'inertia;
        }

        ctx.set_block_at::<1>(AppContext::DRAG_LATCHED, [1])?;

        let mut velocity = get_touch_x(ctx)?.wrapping_sub(get_touch_prev_x(ctx)?);

        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, velocity)?;

        if ops::div_100(ctx.i32_at(AppContext::CAMERA_ZOOM)? as i64) as i32
            > ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?
        {
            let step = get_touch_x(ctx)?.wrapping_sub(get_touch_prev_x(ctx)?);
            let step = step.wrapping_add(step);

            ctx.set_i32_at(
                AppContext::CAMERA_X,
                ctx.i32_at(AppContext::CAMERA_X)?
                    .wrapping_sub(step.wrapping_mul(5)),
            )?;

            velocity = ctx.i32_at(AppContext::SWIPE_VELOCITY)?;
        }

        if velocity != 0 {
            ctx.set_block_at::<1>(AppContext::CAMERA_DRAGGING, [1])?;
        }
    }

    if ops::div_100(ctx.i32_at(AppContext::CAMERA_ZOOM)? as i64) as i32
        > ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?
        && ctx.u8_at(AppContext::CAMERA_DRAGGING)? == 0
    {
        let velocity = ctx.i32_at(AppContext::SWIPE_VELOCITY)?;

        ctx.set_i32_at(
            AppContext::CAMERA_X,
            ctx.i32_at(AppContext::CAMERA_X)?
                .wrapping_sub(velocity.wrapping_mul(5)),
        )?;
    }

    Ok(())
}
