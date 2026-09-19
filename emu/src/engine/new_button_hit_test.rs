use crate::Fault;

use super::{
    AppContext, Button, back_pressed, get_touch_x, get_touch_y, std_function_new_button_invoke,
    touch_is_down, touch_released, ui_node_set_scale, web_view_is_open,
};

const SITE: &str = "new_button_hit_test";

pub fn new_button_hit_test(ctx: &mut AppContext, id: i32, busy: u8) -> Result<(), Fault> {
    let this: &mut Button = ctx
        .buttons
        .buttons
        .get_mut(&id)
        .and_then(|slot| slot.as_deref_mut())
        .ok_or(Fault::NullPointer { site: SITE })?;

    if this.enabled == 0 {
        return Ok(());
    }

    let state = this.state;

    'cancel: {
        if state == 1 {
            if this.touching != 0 {
                this.touching = 0;

                let handler = this.handler;

                std_function_new_button_invoke(ctx, handler, id, 1)?;

                if ctx
                    .buttons
                    .buttons
                    .get_mut(&id)
                    .and_then(|slot| slot.as_deref_mut())
                    .ok_or(Fault::NullPointer { site: SITE })?
                    .state
                    != 1
                {
                    return Ok(());
                }
            }

            let this: &mut Button = ctx
                .buttons
                .buttons
                .get_mut(&id)
                .and_then(|slot| slot.as_deref_mut())
                .ok_or(Fault::NullPointer { site: SITE })?;

            let finished = if this.animated != 0 {
                let frame = this.frame;
                let scale = *this.press_scales.get(frame as i64 as usize).ok_or(
                    Fault::IndexOutOfRange {
                        site: SITE,
                        index: frame as i64,
                        limit: this.press_scales.len() as i64,
                    },
                )?;

                ui_node_set_scale(
                    this.node
                        .as_deref_mut()
                        .ok_or(Fault::NullPointer { site: SITE })?,
                    scale,
                    scale,
                );

                this.animated == 0
            } else {
                true
            };

            let next = (this.frame as i64).wrapping_add(1);

            this.frame = next as i32;

            if (this.press_scales.len() as u64).wrapping_sub(1) >= next as u64 && !finished {
                return Ok(());
            }

            this.state = 0;

            let handler = this.handler;

            return std_function_new_button_invoke(ctx, handler, id, 4);
        }

        if state != 0 {
            return Ok(());
        }

        if this.touchable == 0 || this.pressable == 0 {
            break 'cancel;
        }

        'inside: {
            'outside: {
                if get_touch_x(ctx)? < {
                    let this = ctx
                        .buttons
                        .buttons
                        .get_mut(&id)
                        .and_then(|slot| slot.as_deref_mut())
                        .ok_or(Fault::NullPointer { site: SITE })?;
                    this.offset_x.wrapping_add(this.x)
                } {
                    break 'outside;
                }

                if get_touch_x(ctx)? >= {
                    let this = ctx
                        .buttons
                        .buttons
                        .get_mut(&id)
                        .and_then(|slot| slot.as_deref_mut())
                        .ok_or(Fault::NullPointer { site: SITE })?;
                    this.offset_x.wrapping_add(this.x).wrapping_add(this.width)
                } {
                    break 'outside;
                }

                if get_touch_y(ctx)? < {
                    let this = ctx
                        .buttons
                        .buttons
                        .get_mut(&id)
                        .and_then(|slot| slot.as_deref_mut())
                        .ok_or(Fault::NullPointer { site: SITE })?;
                    this.offset_y.wrapping_add(this.y)
                } {
                    break 'outside;
                }

                if get_touch_y(ctx)? < {
                    let this = ctx
                        .buttons
                        .buttons
                        .get_mut(&id)
                        .and_then(|slot| slot.as_deref_mut())
                        .ok_or(Fault::NullPointer { site: SITE })?;
                    this.offset_y.wrapping_add(this.y).wrapping_add(this.height)
                } {
                    break 'inside;
                }
            }

            if ctx
                .buttons
                .buttons
                .get_mut(&id)
                .and_then(|slot| slot.as_deref_mut())
                .ok_or(Fault::NullPointer { site: SITE })?
                .back_key
                == 0
            {
                break 'cancel;
            }

            if back_pressed(ctx)? == 0 {
                break 'cancel;
            }

            if web_view_is_open(ctx)? {
                break 'cancel;
            }
        }

        if busy != 0 {
            break 'cancel;
        }

        let activate = touch_released(ctx)? != 0
            || (ctx
                .buttons
                .buttons
                .get_mut(&id)
                .and_then(|slot| slot.as_deref_mut())
                .ok_or(Fault::NullPointer { site: SITE })?
                .back_key
                != 0
                && back_pressed(ctx)? != 0);

        if activate {
            let this = ctx
                .buttons
                .buttons
                .get_mut(&id)
                .and_then(|slot| slot.as_deref_mut())
                .ok_or(Fault::NullPointer { site: SITE })?;

            this.state = 1;
            this.frame = 0;

            let handler = this.handler;

            return std_function_new_button_invoke(ctx, handler, id, 3);
        }

        if touch_is_down(ctx)? == 0 {
            break 'cancel;
        }

        let this = ctx
            .buttons
            .buttons
            .get_mut(&id)
            .and_then(|slot| slot.as_deref_mut())
            .ok_or(Fault::NullPointer { site: SITE })?;

        if this.touching != 0 {
            let handler = this.handler;

            return std_function_new_button_invoke(ctx, handler, id, 2);
        }

        this.touching = 1;

        let handler = this.handler;

        return std_function_new_button_invoke(ctx, handler, id, 0);
    }

    let this = ctx
        .buttons
        .buttons
        .get_mut(&id)
        .and_then(|slot| slot.as_deref_mut())
        .ok_or(Fault::NullPointer { site: SITE })?;

    if this.touching != 0 {
        this.touching = 0;

        let handler = this.handler;

        std_function_new_button_invoke(ctx, handler, id, 1)?;
    }

    Ok(())
}
