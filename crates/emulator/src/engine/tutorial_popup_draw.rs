use crate::{Fault, ops};

use super::{
    AppContext, BUTTON_PRESS_BOUNCE, POPUP_GROW_TABLE, Surface, draw_context, draw_cut_rotated,
    draw_cut_scaled, draw_deck, draw_surface_aligned, fill_rect, get_design_height2, get_drawable_width,
    get_scene_id, hit_test_rect, set_alpha, set_color, set_flip, set_tint, sin_deg, touch_is_down,
    ui_sheet_cached,
};

pub fn tutorial_popup_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    let scene = get_scene_id(ctx)?;

    if scene == 0x12c {
        if ctx.i32_at(AppContext::TUTORIAL_DECK_SEEN)? == 1 || ctx.i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN)? == 1 {
            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xcc);
            fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);
            draw_deck(ctx, 0)?;
            set_alpha(draw_context(&mut ctx.draw)?, 0xff);
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? > 9 || ctx.i32_at(AppContext::TUTORIAL_PAGE)? > 0 {
                let arrow = ctx.img039_sheet.clone();
                let arrow = arrow.as_deref().ok_or(Fault::null_pointer())?;
                let width = get_drawable_width(ctx)?;
                let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?;

                draw_cut_scaled(draw_context(&mut ctx.draw)?, arrow, ops::div_2(width).wrapping_sub(0x30), 0x114i32.wrapping_add(bob), 0x60, 0x60, 0);
                set_flip(draw_context(&mut ctx.draw)?, 0);

                if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0xa {
                    let ok = ctx.img101_sheet.clone();
                    let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
                    let press = ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)? as i64 as usize;
                    let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
                    let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x267i32.wrapping_sub(ops::div_2(bounce)) as f64);
                    let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0xeei32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0xa8), bounce.wrapping_add(0x48), 0);

                    let label = ctx.img006_sheet.clone();
                    let label = label.as_deref().ok_or(Fault::null_pointer())?;
                    let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
                    let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x28bi32.wrapping_sub(ops::div_2(bounce)) as f64);
                    let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0xf6i32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0x61), bounce.wrapping_add(0x37), 0x14);

                    if touch_is_down(ctx)? != 0
                        && hit_test_rect(
                            ctx,
                            ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                            ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                            ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                            ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
                        )?
                        && ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0
                    {
                        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 614.0);
                        let ticks = ctx.i32_at(AppContext::TUTORIAL_PRESS_TICKS)?;
                        let lead = if ticks >= 0 { ticks as i8 } else { (ticks as i8).wrapping_add(3) };
                        let phase = (ticks as i8).wrapping_sub(lead & !3);
                        let cut = ((phase as u8 >> 7) as i8).wrapping_add(phase) >> 1;

                        draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0xed, 0xa8, 0x48, cut.wrapping_add(1) as u8 as i32);
                    }
                }
            }

            let panel = ctx.scene_img005_sheet.clone();
            let panel = panel.as_deref().ok_or(Fault::null_pointer())?;
            let width = get_drawable_width(ctx)?;
            let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)? as i64 as usize;
            let grow = *POPUP_GROW_TABLE.get(timer).ok_or(Fault::index_out_of_range(timer as i64, POPUP_GROW_TABLE.len() as i64))?;

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                panel,
                ops::div_neg_200(grow.wrapping_mul(0x2b2)).wrapping_add(ops::div_2(width)),
                ops::div_neg_200(grow.wrapping_mul(0xb3)).wrapping_add(0x8c),
                ops::div_100(grow.wrapping_mul(0x2b2)),
                ops::div_100(grow.wrapping_mul(0xb3)),
                0,
            );

            if (ctx.i32_at(AppContext::TUTORIAL_TIMER)? as u32) < 4 {
                return Ok(());
            }

            let mut row = -2i32;

            if ctx.tutorial_lines[3].is_none() {
                row = if ctx.tutorial_lines[2].is_some() { -1 } else { ctx.tutorial_lines[1].is_none() as i32 };
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

            if let Some(line) = ctx.tutorial_lines[0].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x6b), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[1].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x8f), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[2].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0xb3), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[3].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0xd7), 1);
            }

            return Ok(());
        }

        if ctx.i32_at(AppContext::TUTORIAL_CAT_GOD_SEEN)? == 1 {
            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xcc);
            fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);

            let statue = ctx.img002_sheet.clone();
            let statue = statue.as_deref().ok_or(Fault::null_pointer())?;
            let width = get_drawable_width(ctx)?;
            let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)? as i64 as usize;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let half = ops::div_2(bounce);
            let sink = ctx.i32_at(AppContext::CAT_GOD_BUTTON_SINK)?;
            let spin = ctx.f32_at(AppContext::CAT_GOD_SPIN)?;

            draw_cut_rotated(
                draw_context(&mut ctx.draw)?,
                statue,
                ops::div_2(width).wrapping_sub(half).wrapping_sub(0x41),
                (-0x18i32).wrapping_sub(shift.wrapping_add(sink).wrapping_add(half)),
                bounce.wrapping_add(0x83),
                bounce.wrapping_add(0x83),
                spin,
                0,
                0,
                0,
                5,
                0x28,
            );

            let width = get_drawable_width(ctx)?;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let half = ops::div_2(bounce);

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                statue,
                ops::div_2(width).wrapping_sub(half).wrapping_sub(0x26),
                1i32.wrapping_sub(sink.wrapping_add(shift).wrapping_add(half)),
                bounce.wrapping_add(0x4c),
                bounce.wrapping_add(0x4c),
                0x29,
            );

            if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0xa {
                set_flip(draw_context(&mut ctx.draw)?, 2);

                let arrow = ctx.img039_sheet.clone();
                let arrow = arrow.as_deref().ok_or(Fault::null_pointer())?;
                let width = get_drawable_width(ctx)?;
                let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?.wrapping_mul(0x1e);
                let y = ops::cvttss2si(sin_deg(bob as f32) * -10.0 + 0x61i32.wrapping_sub(shift) as f32);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, arrow, ops::div_2(width).wrapping_sub(0x30), y, 0x60, 0x60, 0);
                set_flip(draw_context(&mut ctx.draw)?, 0);
                let ok = ctx.img101_sheet.clone();
                let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
                let press = ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)? as i64 as usize;
                let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x267i32.wrapping_sub(ops::div_2(bounce)) as f64);
                let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

                draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x184i32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0xa8), bounce.wrapping_add(0x48), 0);

                let label = ctx.img006_sheet.clone();
                let label = label.as_deref().ok_or(Fault::null_pointer())?;
                let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x28bi32.wrapping_sub(ops::div_2(bounce)) as f64);
                let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

                draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0x18ci32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0x61), bounce.wrapping_add(0x37), 0x14);

                if touch_is_down(ctx)? != 0
                    && hit_test_rect(
                        ctx,
                        ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                        ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                        ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                        ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
                    )?
                    && ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0
                {
                    let ok = ctx.img101_sheet.clone();
                    let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
                    let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 614.0);
                    let ticks = ctx.i32_at(AppContext::TUTORIAL_PRESS_TICKS)?;
                    let lead = if ticks >= 0 { ticks as i8 } else { (ticks as i8).wrapping_add(3) };
                    let phase = (ticks as i8).wrapping_sub(lead & !3);
                    let cut = ((phase as u8 >> 7) as i8).wrapping_add(phase) >> 1;

                    draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x183, 0xa8, 0x48, cut.wrapping_add(1) as u8 as i32);
                }
            }

            let panel = ctx.scene_img005_sheet.clone();
            let panel = panel.as_deref().ok_or(Fault::null_pointer())?;
            let width = get_drawable_width(ctx)?;
            let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)? as i64 as usize;
            let grow = *POPUP_GROW_TABLE.get(timer).ok_or(Fault::index_out_of_range(timer as i64, POPUP_GROW_TABLE.len() as i64))?;

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                panel,
                ops::div_neg_200(grow.wrapping_mul(0x2b2)).wrapping_add(ops::div_2(width)),
                ops::div_neg_200(grow.wrapping_mul(0xb3)).wrapping_add(0x122),
                ops::div_100(grow.wrapping_mul(0x2b2)),
                ops::div_100(grow.wrapping_mul(0xb3)),
                0,
            );

            if (ctx.i32_at(AppContext::TUTORIAL_TIMER)? as u32) >= 4 {
                let mut row = -2i32;

                if ctx.tutorial_lines[3].is_none() {
                    row = if ctx.tutorial_lines[2].is_some() { -1 } else { ctx.tutorial_lines[1].is_none() as i32 };
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

                if let Some(line) = ctx.tutorial_lines[0].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x101), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[1].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x125), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[2].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x149), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[3].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x16d), 1);
                }
            }

            return Ok(());
        }

        if ctx.i32_at(AppContext::SHOP_TUTORIAL_SEEN)? != 1 {
            if ctx.i32_at(AppContext::CAT_GOD_INTRO_STEP)? != 1 {
                return Ok(());
            }

            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xcc);
            fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);

            if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0xa {
                let arrow = ctx.img039_sheet.clone();
                let arrow = arrow.as_deref().ok_or(Fault::null_pointer())?;
                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 786.0);
                let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?.wrapping_mul(0x1e);
                let y = ops::cvttss2si(sin_deg(bob as f32) * 10.0 + 235.0);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, arrow, x, y, 0x60, 0x60, 0);
            }

            let button = ctx.img042_sheet.clone();
            let button = button.as_deref().ok_or(Fault::null_pointer())?;
            let press = ctx.i32_at(AppContext::CAT_GOD_INTRO_BUTTON_PRESS)? as i64 as usize;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x312i32.wrapping_sub(ops::div_2(bounce)) as f64);
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let offset = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?;

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                button,
                x,
                0i32.wrapping_sub(ops::div_2(bounce)).wrapping_add(offset).wrapping_add(0x159),
                bounce.wrapping_add(0x60),
                bounce.wrapping_add(0x60),
                3,
            );
            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

            let label = ctx.label_texts.get(6).copied().flatten().ok_or(Fault::null_pointer())?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 834.0);
            let offset = ctx.i32_at(AppContext::CAT_GOD_OFFSET)?;

            draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&label), x, 0x1f9i32.wrapping_add(offset), 1);
            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
            let panel = ctx.scene_img005_sheet.clone();
            let panel = panel.as_deref().ok_or(Fault::null_pointer())?;
            let width = get_drawable_width(ctx)?;
            let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)? as i64 as usize;
            let grow = *POPUP_GROW_TABLE.get(timer).ok_or(Fault::index_out_of_range(timer as i64, POPUP_GROW_TABLE.len() as i64))?;

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                panel,
                ops::div_neg_200(grow.wrapping_mul(0x2b2)).wrapping_add(ops::div_2(width)),
                ops::div_neg_200(grow.wrapping_mul(0xb3)).wrapping_add(0x122),
                ops::div_100(grow.wrapping_mul(0x2b2)),
                ops::div_100(grow.wrapping_mul(0xb3)),
                0,
            );

            if (ctx.i32_at(AppContext::TUTORIAL_TIMER)? as u32) >= 4 {
                let mut row = -2i32;

                if ctx.tutorial_lines[3].is_none() {
                    row = if ctx.tutorial_lines[2].is_some() { -1 } else { ctx.tutorial_lines[1].is_none() as i32 };
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

                if let Some(line) = ctx.tutorial_lines[0].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x101), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[1].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x125), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[2].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x149), 1);
                }

                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                if let Some(line) = ctx.tutorial_lines[3].clone() {
                    let x = ops::div_2(get_drawable_width(ctx)?);

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x16d), 1);
                }
            }

            if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? < 0xa {
                return Ok(());
            }

            let ok = ctx.img101_sheet.clone();
            let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
            let press = ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)? as i64 as usize;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x267i32.wrapping_sub(ops::div_2(bounce)) as f64);
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x184i32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0xa8), bounce.wrapping_add(0x48), 0);

            let label = ctx.img006_sheet.clone();
            let label = label.as_deref().ok_or(Fault::null_pointer())?;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x28bi32.wrapping_sub(ops::div_2(bounce)) as f64);
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0x18ci32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0x61), bounce.wrapping_add(0x37), 0x14);

            if touch_is_down(ctx)? == 0 {
                return Ok(());
            }

            if !hit_test_rect(
                ctx,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
            )? || ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0
            {
                return Ok(());
            }

            let ok = ctx.img101_sheet.clone();
            let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 614.0);
            let ticks = ctx.i32_at(AppContext::TUTORIAL_PRESS_TICKS)?;
            let lead = if ticks >= 0 { ticks as i8 } else { (ticks as i8).wrapping_add(3) };
            let phase = (ticks as i8).wrapping_sub(lead & !3);
            let cut = ((phase as u8 >> 7) as i8).wrapping_add(phase) >> 1;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x183, 0xa8, 0x48, cut.wrapping_add(1) as u8 as i32);

            return Ok(());
        }

        let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xcc);
        fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);
        let panel = ctx.scene_img005_sheet.clone();
        let panel = panel.as_deref().ok_or(Fault::null_pointer())?;
        let width = get_drawable_width(ctx)?;
        let timer = ctx.i32_at(AppContext::TUTORIAL_TIMER)? as i64 as usize;
        let grow = *POPUP_GROW_TABLE.get(timer).ok_or(Fault::index_out_of_range(timer as i64, POPUP_GROW_TABLE.len() as i64))?;

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            panel,
            ops::div_neg_200(grow.wrapping_mul(0x2b2)).wrapping_add(ops::div_2(width)),
            ops::div_neg_200(grow.wrapping_mul(0xb3)).wrapping_add(0x122),
            ops::div_100(grow.wrapping_mul(0x2b2)),
            ops::div_100(grow.wrapping_mul(0xb3)),
            0,
        );

        if (ctx.i32_at(AppContext::TUTORIAL_TIMER)? as u32) >= 4 {
            let mut row = -2i32;

            if ctx.tutorial_lines[3].is_none() {
                row = if ctx.tutorial_lines[2].is_some() { -1 } else { ctx.tutorial_lines[1].is_none() as i32 };
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

            if let Some(line) = ctx.tutorial_lines[0].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x101), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[1].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x125), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[2].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x149), 1);
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if let Some(line) = ctx.tutorial_lines[3].clone() {
                let x = ops::div_2(get_drawable_width(ctx)?);

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line), x, row.wrapping_mul(9).wrapping_mul(2).wrapping_add(0x16d), 1);
            }
        }

        if ctx.i32_at(AppContext::TUTORIAL_POPUP_FRAME)? >= 0xa {
            let arrow = ctx.img039_sheet.clone();
            let arrow = arrow.as_deref().ok_or(Fault::null_pointer())?;
            let x = get_drawable_width(ctx)?.wrapping_sub(0x125);
            let bob = ctx.i32_at(AppContext::TUTORIAL_BOB_PHASE)?.wrapping_mul(0x1e);
            let y = ops::cvttss2si(sin_deg(bob as f32) * 10.0 + 470.0);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, arrow, x, y, 0x60, 0x60, 0);
            let ok = ctx.img101_sheet.clone();
            let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
            let press = ctx.i32_at(AppContext::CAT_GOD_GLOW_TIMER)? as i64 as usize;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x267i32.wrapping_sub(ops::div_2(bounce)) as f64);
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x184i32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0xa8), bounce.wrapping_add(0x48), 0);

            let label = ctx.img006_sheet.clone();
            let label = label.as_deref().ok_or(Fault::null_pointer())?;
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 0x28bi32.wrapping_sub(ops::div_2(bounce)) as f64);
            let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, label, x, 0x18ci32.wrapping_sub(ops::div_2(bounce)), bounce.wrapping_add(0x61), bounce.wrapping_add(0x37), 0x14);

            if touch_is_down(ctx)? != 0
                && hit_test_rect(
                    ctx,
                    ctx.i32_at(AppContext::TUTORIAL_BUTTON_X)?,
                    ctx.i32_at(AppContext::TUTORIAL_BUTTON_Y)?,
                    ctx.i32_at(AppContext::TUTORIAL_BUTTON_W)?,
                    ctx.i32_at(AppContext::TUTORIAL_BUTTON_H)?,
                )?
                && ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0
            {
                let ok = ctx.img101_sheet.clone();
                let ok = ok.as_deref().ok_or(Fault::null_pointer())?;
                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_sub(0x3c0) as f64 * 0.5 + 614.0);
                let ticks = ctx.i32_at(AppContext::TUTORIAL_PRESS_TICKS)?;
                let lead = if ticks >= 0 { ticks as i8 } else { (ticks as i8).wrapping_add(3) };
                let phase = (ticks as i8).wrapping_sub(lead & !3);
                let cut = ((phase as u8 >> 7) as i8).wrapping_add(phase) >> 1;

                draw_cut_scaled(draw_context(&mut ctx.draw)?, ok, x, 0x183, 0xa8, 0x48, cut.wrapping_add(1) as u8 as i32);
            }
        }

        let icon = ui_sheet_cached(ctx, 0x15)?.ok_or(Fault::null_pointer())?;
        let width = get_drawable_width(ctx)?;
        let press = ctx.i32_at(AppContext::LOSE_SHOP_PRESS)? as i64 as usize;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
        let half = ops::div_2(bounce);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, &icon, width.wrapping_sub(half).wrapping_sub(0x110), 0x251i32.wrapping_sub(half), bounce.wrapping_add(0x37), bounce.wrapping_add(0x2a), 0x15);

        let icon = ui_sheet_cached(ctx, 0x15)?.ok_or(Fault::null_pointer())?;
        let width = get_drawable_width(ctx)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
        let half = ops::div_2(bounce);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, &icon, width.wrapping_sub(half).wrapping_sub(0xe3), 0x262i32.wrapping_sub(half), bounce.wrapping_add(0x1b), bounce.wrapping_add(0x1a), 0x12);

        return Ok(());
    }

    if scene == 0x64 {
        ctx.scene_host().ok_or(Fault::host_missing())?.draw_map_tutorial();
    }

    Ok(())
}
