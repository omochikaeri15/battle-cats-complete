use crate::{Fault, ops};

use super::{
    AppContext, atan2_deg, back_pressed, deck_row_swap_tick, get_drawable_width, get_text_texture,
    get_touch_start_x, get_touch_start_y, get_touch_x, get_touch_y, hit_test_rect, play_sound,
    set_bgm_duck, sound_manager, std_string_equals, text_texture_cache, touch_is_down, touch_released,
};

pub fn tutorial_popup_update(ctx: &mut AppContext) -> Result<(), Fault> {
    let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)?;

    ctx.set_i32_at(AppContext::TUTORIAL_TIMER, timer.wrapping_add(1))?;

    if timer == 0 {
        play_sound(sound_manager(ctx)?, 0x2a, None);
    }

    ctx.set_i32_at(AppContext::TUTORIAL_PRESS_TICKS, ctx.i32_at(AppContext::TUTORIAL_PRESS_TICKS)?.wrapping_add(1))?;

    if ctx.i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN)? == 1 {
        ctx.set_f32_at(AppContext::CAT_GOD_SPIN, ctx.f32_at(AppContext::CAT_GOD_SPIN)? + 0.5)?;
    }

    if ctx.i32_at(AppContext::TUTORIAL_DECK_SEEN)? == 1 {
        if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0x14 {
            let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?;

            ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, bob.wrapping_add(4))?;

            if bob >= 0x60 {
                ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, 0x64)?;

                if ctx.i32_at(AppContext::TUTORIAL_PAGE)? == 0 {
                    if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
                        ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                        ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 1)?;
                    } else {
                        let direction = ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)?;

                        if direction == -1 {
                            if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 {
                                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                            }
                        } else if direction == 1 && ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 {
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?;
                        }
                    }

                    ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
                    ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
                    ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [1])?;
                }
            }
        }

        if ctx.i32_at(AppContext::TUTORIAL_PAGE)? == 1 {
            if touch_is_down(ctx)? == 0 {
                ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [0])?;
                ctx.set_block_at::<1>(AppContext::CAMERA_DRAGGING, [0])?;
            } else if ctx.u8_at(AppContext::DECK_SWIPE_LATCHED)? | ctx.u8_at(AppContext::PINCH_ZOOMED)? == 0 {
                ctx.set_i32_at(AppContext::SWIPE_DY, get_touch_y(ctx)?.wrapping_sub(get_touch_start_y(ctx)?))?;

                let angle = ops::cvttss2si(atan2_deg(
                    get_touch_y(ctx)?.wrapping_sub(get_touch_start_y(ctx)?) as f32,
                    get_touch_x(ctx)?.wrapping_sub(get_touch_start_x(ctx)?) as f32,
                ));

                ctx.set_i32_at(AppContext::SWIPE_ANGLE, angle)?;

                if ctx.u8_at(AppContext::DECK_BACK_ROW_ENABLED)? != 0 && ctx.u8_at(AppContext::CAMERA_DRAGGING)? == 0 {
                    let dy = ctx.i32_at(AppContext::SWIPE_DY)?;

                    if dy <= -0x3c && angle >= 0xe1 && angle <= 0x13b {
                        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;

                        if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 0)?;
                        } else {
                            let direction = ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)?;

                            if direction == -1 {
                                if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 {
                                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                                }
                            } else if direction == 1 && ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 {
                                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?;
                            }
                        }

                        ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
                        ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [1])?;
                    } else if dy >= 0x3c && (angle.wrapping_sub(0x2d) as u32) <= 0x5a {
                        if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 1)?;
                        } else {
                            let direction = ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)?;

                            if direction == -1 {
                                if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 {
                                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                                }
                            } else if direction == 1 && ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 {
                                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?;
                            }
                        }

                        ctx.set_i32_at(AppContext::SWIPE_VELOCITY, 0)?;
                        ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
                        ctx.set_block_at::<1>(AppContext::DECK_SWIPE_LATCHED, [1])?;
                    }
                }
            }
        }

        let target = ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)?;

        deck_row_swap_tick(ctx, target)?;

        if ctx.i32_at(AppContext::TUTORIAL_PAGE)? != 0 || ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
            if ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)? >= 0x64 {
                ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, 0)?;
            }
        }
    } else if ctx.i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN)? != 1 {
        ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?.wrapping_add(1))?;
    } else {
        if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0x14 {
            let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?;
            let next = if bob < 0x60 { bob.wrapping_add(4) } else { 0x64 };

            ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, next)?;
        }

        if ctx.i32_at(AppContext::TUTORIAL_PAGE)? != 0 || ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
            if ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)? >= 0x64 {
                ctx.set_i32_at(AppContext::TUTORIAL_BOB_PHASE, 0)?;
            }
        }
    }

    let mut timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)?;

    if timer == 1 {
        ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_X, ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 615.0))?;
        ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_Y, 0x17c)?;
        ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_W, 0xa8)?;
        ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_H, 0x58)?;
        ctx.set_block_at::<1>(AppContext::TUTORIAL_BUTTON_HELD, [0])?;
        ctx.set_i32_at(AppContext::CAT_GOD_GLOW_TIMER, 0)?;

        for line in ctx.tutorial_lines.iter_mut() {
            *line = None;
        }

        if ctx.i32_at(AppContext::TUTORIAL_DECK_SEEN)? == 1 {
            ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_Y, 0xe6)?;

            let mut line = 0usize;

            while line != 4 {
                let flat = 3usize * 12 + (ctx.i32_at(AppContext::TUTORIAL_PAGE)? as i64 as usize) * 4 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN)? == 1 {
            ctx.set_i32_at(AppContext::TUTORIAL_BUTTON_Y, 0xe6)?;

            let mut line = 0usize;

            while line != 4 {
                let flat = 24usize * 12 + (ctx.i32_at(AppContext::TUTORIAL_PAGE)? as i64 as usize) * 4 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                let flat = 4usize * 12 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_FORMATION_SEEN)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                let flat = 6usize * 12 + (ctx.i32_at(AppContext::TUTORIAL_PAGE)? as i64 as usize) * 4 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::SHOP_TUTORIAL_SEEN)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                let flat = 8usize * 12 + (ctx.i32_at(AppContext::TUTORIAL_PAGE)? as i64 as usize) * 4 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                let flat = 9usize * 12 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"\xef\xbc\xa0") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_SCENE_JUMP_SEEN)? == 1 {
            ctx.set_i32_at(AppContext::TUTORIAL_SCENE_JUMP_SEEN, 2)?;
            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;

            if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                set_bgm_duck(sound_manager(ctx)?, 0x64);
            }

            ctx.set_i32_at(AppContext::TUTORIAL_EXIT_STATE, 2)?;
            ctx.set_i32_at(AppContext::PENDING_SCENE, 0x11)?;
        } else if ctx.i32_at(AppContext::TUTORIAL_MISSION_SEEN)? == 1 {
            ctx.set_i32_at(AppContext::TUTORIAL_MISSION_SEEN, 2)?;
            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;

            if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? != 0 {
                ctx.set_i32_at(AppContext::TUTORIAL_EXIT_STATE, 2)?;
            } else {
                set_bgm_duck(sound_manager(ctx)?, 0x64);
                ctx.set_i32_at(AppContext::TUTORIAL_EXIT_STATE, 2)?;
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_ROW12_SEEN)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                if ctx.tutorial_lines[line].is_none() {
                    let flat = 12usize * 12 + line;
                    let text = ctx
                        .tutorial_pages
                        .get(flat / 12)
                        .and_then(|page| page.get(flat % 12))
                        .cloned()
                        .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                    if std_string_equals(&text, b"") {
                        break;
                    }

                    let font = ctx.default_font.clone();

                    ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                }

                line += 1;
            }

            ctx.set_i32_at(AppContext::TUTORIAL_ROW12_SEEN, 2)?;
            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;

            if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                set_bgm_duck(sound_manager(ctx)?, 0x64);
            }

            ctx.set_i32_at(AppContext::TUTORIAL_EXIT_STATE, 2)?;
        } else if ctx.i32_at(AppContext::TUTORIAL_ROW10_SEEN)? == 1 {
            let mut line = 0usize;

            while line != 4 {
                let flat = 10usize * 12 + line;
                let text = ctx
                    .tutorial_pages
                    .get(flat / 12)
                    .and_then(|page| page.get(flat % 12))
                    .cloned()
                    .ok_or(Fault::index_out_of_range(flat as i64, ctx.tutorial_pages.len() as i64 * 12))?;

                if std_string_equals(&text, b"") {
                    break;
                }

                let font = ctx.default_font.clone();

                ctx.tutorial_lines[line] = Some(get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0));
                line += 1;
            }

            ctx.set_i32_at(AppContext::TUTORIAL_ROW10_SEEN, 2)?;
            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;

            if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                set_bgm_duck(sound_manager(ctx)?, 0x64);
            }

            ctx.set_i32_at(AppContext::TUTORIAL_EXIT_STATE, 2)?;
        }

        timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)?;
    }

    let mut settled = false;

    if (timer as u32) > 4 {
        ctx.set_i32_at(AppContext::TUTORIAL_TIMER, 4)?;

        let frame = ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)?.wrapping_add(1);

        ctx.set_i32_at(AppContext::TUTORIAL_POPUP_FRAME, frame)?;
        settled = frame >= 0x14;
    } else if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0x14 {
        settled = true;
    }

    if !settled {
        return Ok(());
    }

    let press = ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)?;

    if press <= 0 {
        let down = touch_is_down(ctx)?;
        let mut hit = false;

        if down != 0 {
            hit = hit_test_rect(
                ctx,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
            )?;
        }

        if down == 0 || !hit {
            ctx.set_block_at::<1>(AppContext::TUTORIAL_BUTTON_HELD, [0])?;
        } else if ctx.u8_at(AppContext::TUTORIAL_BUTTON_HELD)? == 0 {
            ctx.set_block_at::<1>(AppContext::TUTORIAL_BUTTON_HELD, [1])?;
            play_sound(sound_manager(ctx)?, 0xa, None);
        }

        let released = touch_released(ctx)?;
        let mut hit = false;

        if released != 0 {
            hit = hit_test_rect(
                ctx,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
            )?;
        }

        let mut back = 0u8;

        if released == 0 || !hit {
            back = back_pressed(ctx)?;
        }

        if (released != 0 && hit) || back != 0 {
            ctx.set_i32_at(AppContext::CAT_GOD_GLOW_TIMER, ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)?.wrapping_add(1))?;
            play_sound(sound_manager(ctx)?, 0xb, None);
        }

        return Ok(());
    }

    ctx.set_i32_at(AppContext::CAT_GOD_GLOW_TIMER, press.wrapping_add(1))?;

    if (press as u32) < 5 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::CAT_GOD_GLOW_TIMER, 0)?;
    ctx.set_block_at::<0x10>(AppContext::TUTORIAL_TIMER, [0; 0x10])?;

    for line in ctx.tutorial_lines.iter_mut() {
        *line = None;
    }

    if ctx.i32_at(AppContext::TUTORIAL_DECK_SEEN)? == 1 {
        ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

        if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
            set_bgm_duck(sound_manager(ctx)?, 0x64);
        }

        ctx.set_i32_at(AppContext::TUTORIAL_DECK_SEEN, 2)?;
    } else if ctx.i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN)? == 1 {
        ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

        if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
            set_bgm_duck(sound_manager(ctx)?, 0x64);
        }

        ctx.set_i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN, 2)?;
        ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [0])?;
    } else if ctx.i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN)? == 1 {
        ctx.set_i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN, 2)?;
        ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

        if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
            set_bgm_duck(sound_manager(ctx)?, 0x64);
        }
    } else if ctx.i32_at(AppContext::TUTORIAL_FORMATION_SEEN)? != 1 {
        if ctx.i32_at(AppContext::SHOP_TUTORIAL_SEEN)? != 1 {
            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? == 1 {
                ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, 2)?;
                ctx.set_i32_at(AppContext::CAT_GOD_OPEN_TICKS, ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?)?;
                ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

                if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                    set_bgm_duck(sound_manager(ctx)?, 0x64);
                }
            }
        } else if ctx.i32_at(AppContext::TUTORIAL_PAGE)? == 0 {
            ctx.set_i32_at(AppContext::TUTORIAL_TIMER, 0)?;
            ctx.set_i32_at(AppContext::TUTORIAL_PRESS_TICKS, 0)?;
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 1)?;
        } else {
            ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;
            ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

            if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
                set_bgm_duck(sound_manager(ctx)?, 0x64);
            }

            ctx.set_i32_at(AppContext::SHOP_TUTORIAL_SEEN, 2)?;
        }
    } else if ctx.i32_at(AppContext::TUTORIAL_PAGE)? == 0 {
        ctx.set_i32_at(AppContext::TUTORIAL_TIMER, 0)?;
        ctx.set_i32_at(AppContext::TUTORIAL_PRESS_TICKS, 0)?;
        ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 1)?;
    } else {
        ctx.set_block_at::<1>(AppContext::TUTORIAL_POPUP_OPEN, [0])?;

        if ctx.u8_at(AppContext::BGM_SWITCH_STATE)? == 0 {
            set_bgm_duck(sound_manager(ctx)?, 0x64);
        }

        ctx.set_i32_at(AppContext::TUTORIAL_PAGE, 0)?;
        ctx.set_i32_at(AppContext::TUTORIAL_FORMATION_SEEN, 2)?;
    }

    Ok(())
}
