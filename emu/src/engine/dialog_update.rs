use crate::Fault;

use super::{
    AppContext, back_pressed, dialog_button_rect, hit_test_rect, play_sound, sound_manager,
    touch_released,
};

pub fn dialog_update(ctx: &mut AppContext, dialog: u64, is_top: u8) -> Result<(), Fault> {
    let this = ctx
        .dialogs
        .objects
        .get_mut(&dialog)
        .ok_or(Fault::null_pointer())?;

    match this.state {
        2 => {
            let timer = this.timer.wrapping_add(1);

            this.timer = timer;

            if (timer as u32) < 5 {
                return Ok(());
            }

            this.timer = 0;
            this.state = 3;

            let button = this.button;
            let handler = this.on_event.ok_or(Fault::bad_function_call())?;

            handler(ctx, dialog, 6, button)?;

            let this = ctx
                .dialogs
                .objects
                .get_mut(&dialog)
                .ok_or(Fault::null_pointer())?;
            let button = this.button;
            let handler = this.on_event.ok_or(Fault::bad_function_call())?;

            handler(ctx, dialog, 5, button)?;

            ctx.dialogs
                .objects
                .get_mut(&dialog)
                .ok_or(Fault::null_pointer())?
                .lines
                .clear();

            Ok(())
        }
        1 => {
            this.tick = this.tick.wrapping_add(1);

            let timer = this.timer;

            if timer > 0 {
                this.timer = timer.wrapping_add(1);

                if (timer as u32) < 4 {
                    return Ok(());
                }

                this.timer = 0;

                let button = this.button;
                let handler = this.on_event.ok_or(Fault::bad_function_call())?;

                return handler(ctx, dialog, 6, button);
            }

            if is_top == 0 {
                return Ok(());
            }

            if this.kind as u32 > 3 {
                return Ok(());
            }

            if ctx.u8_at(AppContext::TOUCH_DOWN_LATCH)? != 0 {
                let mut slot = 0u64;

                loop {
                    let this = ctx
                        .dialogs
                        .objects
                        .get(&dialog)
                        .ok_or(Fault::null_pointer())?;
                    let last = this.kind.wrapping_sub(1) as u32;

                    if last > 2 {
                        break;
                    }

                    if slot > last as u64 {
                        break;
                    }

                    let rect = dialog_button_rect(this, slot as i32);
                    let hit = hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])?;
                    let this = ctx
                        .dialogs
                        .objects
                        .get_mut(&dialog)
                        .ok_or(Fault::null_pointer())?;

                    if hit {
                        if this.hover[slot as usize] == 0 && this.audible[slot as usize] != 0 {
                            play_sound(sound_manager(ctx)?, 0xa, None);

                            ctx.dialogs
                                .objects
                                .get_mut(&dialog)
                                .ok_or(Fault::null_pointer())?
                                .hover[slot as usize] = 1;
                        }
                    } else {
                        this.hover[slot as usize] = 0;
                    }

                    slot += 1;
                }

                let this = ctx
                    .dialogs
                    .objects
                    .get(&dialog)
                    .ok_or(Fault::null_pointer())?;

                if this.flags & 8 != 0 {
                    let rect = dialog_button_rect(this, -2);

                    if hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                        if this.close_hover == 0 {
                            play_sound(sound_manager(ctx)?, 0xa, None);

                            ctx.dialogs
                                .objects
                                .get_mut(&dialog)
                                .ok_or(Fault::null_pointer())?
                                .close_hover = 1;
                        }
                    } else {
                        ctx.dialogs
                            .objects
                            .get_mut(&dialog)
                            .ok_or(Fault::null_pointer())?
                            .close_hover = 0;
                    }
                }
            } else {
                let this = ctx
                    .dialogs
                    .objects
                    .get_mut(&dialog)
                    .ok_or(Fault::null_pointer())?;

                this.hover = [0; 3];
                this.close_hover = 0;
            }

            'pressed: {
                let back_button = ctx
                    .dialogs
                    .objects
                    .get(&dialog)
                    .ok_or(Fault::null_pointer())?
                    .back_button;

                'tapped: {
                    if touch_released(ctx)? == 0 {
                        if back_button == -3 {
                            break 'pressed;
                        }

                        if back_pressed(ctx)? == 0 {
                            break 'pressed;
                        }
                    }

                    if back_button == -3 {
                        break 'tapped;
                    }

                    if back_pressed(ctx)? == 0 {
                        break 'tapped;
                    }

                    let this = ctx
                        .dialogs
                        .objects
                        .get_mut(&dialog)
                        .ok_or(Fault::null_pointer())?;

                    this.button = this.back_button;
                    this.timer = this.timer.wrapping_add(1);

                    let button = this.button;
                    let code = if button < 0 {
                        2
                    } else if this.visible[button as usize] == 0 {
                        3
                    } else {
                        2i32.wrapping_add((this.lit[button as usize] < 1) as i32)
                    };
                    let handler = this.on_event.ok_or(Fault::bad_function_call())?;

                    handler(ctx, dialog, code, button)?;

                    break 'pressed;
                }

                ctx.dialogs
                    .objects
                    .get_mut(&dialog)
                    .ok_or(Fault::null_pointer())?
                    .button = -1;

                let mut slot = 0i32;

                loop {
                    let this = ctx
                        .dialogs
                        .objects
                        .get(&dialog)
                        .ok_or(Fault::null_pointer())?;

                    if this.kind.wrapping_sub(1) as u32 > 2 {
                        break;
                    }

                    if this.kind as u32 <= slot as u32 {
                        break;
                    }

                    let rect = dialog_button_rect(this, slot);

                    if hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                        let this = ctx
                            .dialogs
                            .objects
                            .get_mut(&dialog)
                            .ok_or(Fault::null_pointer())?;

                        this.button = slot;
                        this.timer = this.timer.wrapping_add(1);

                        break;
                    }

                    slot = slot.wrapping_add(1);
                }

                let this = ctx
                    .dialogs
                    .objects
                    .get(&dialog)
                    .ok_or(Fault::null_pointer())?;

                if this.flags & 8 != 0 {
                    let rect = dialog_button_rect(this, -2);

                    if hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? {
                        let this = ctx
                            .dialogs
                            .objects
                            .get_mut(&dialog)
                            .ok_or(Fault::null_pointer())?;

                        this.button = -2;
                        this.timer = this.timer.wrapping_add(1);
                    }
                }

                let this = ctx
                    .dialogs
                    .objects
                    .get(&dialog)
                    .ok_or(Fault::null_pointer())?;
                let button = this.button;
                let code = if button < 0 {
                    2
                } else if this.visible[button as usize] == 0 {
                    3
                } else {
                    2i32.wrapping_add((this.lit[button as usize] < 1) as i32)
                };
                let handler = this.on_event.ok_or(Fault::bad_function_call())?;

                handler(ctx, dialog, code, button)?;
            }

            let Some(this) = ctx.dialogs.objects.get(&dialog) else {
                return Ok(());
            };

            if this.state != 1 {
                return Ok(());
            }

            if let Some(update) = this.on_update {
                update(ctx, dialog)?;
            }

            Ok(())
        }
        0 => {
            let frame = this.frame;
            let code = if frame as u32 > 3 {
                this.state = 1;

                1
            } else {
                this.frame = frame.wrapping_add(1);

                0
            };
            let handler = this.on_event.ok_or(Fault::bad_function_call())?;

            handler(ctx, dialog, code, -1)
        }
        _ => Ok(()),
    }
}
