use crate::Fault;

use super::{
    AppContext, BUTTON_PRESS_BOUNCE, Surface, draw_context, draw_cut_scaled, draw_surface_aligned,
    fill_polygon, fill_rect_f, find_item_index, get_battle_status, get_drawable_width,
    get_item_description, get_powerup_available, get_right_inset_logical, powerup_available,
    set_draw_origin, set_tint, text_block_draw, touch_is_down,
};

const BLANK_LINE: &[u8] = "\u{ff20}".as_bytes();

pub fn draw_powerup_bar(ctx: &mut AppContext) -> Result<(), Fault> {
    let x = get_right_inset_logical(ctx)?.wrapping_neg();
    let y = ctx
        .i32_at(AppContext::LETTERBOX_PAD)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    set_draw_origin(ctx, x, y)?;

    for column in (0..6i32).rev() {
        if !powerup_available(ctx, column)? {
            continue;
        }

        let press_at = AppContext::SPEED_UP_PRESS + column as usize * 4;
        let (press, shift, grow, top, cut) =
            if ctx.i32_at(AppContext::POWERUPS + column as usize * 4)? == 0 {
                (
                    ctx.i32_at(press_at)?,
                    -0x201,
                    0x3a,
                    0x3a_i32,
                    column.wrapping_add(6),
                )
            } else {
                let latched = ctx.u8_at(AppContext::SPEED_UP_LATCH)?;

                if column == 0 && latched != 0 {
                    (
                        ctx.i32_at(AppContext::SPEED_UP_PRESS)?,
                        -0x20c,
                        0x50,
                        0x2f,
                        0x10,
                    )
                } else {
                    (ctx.i32_at(press_at)?, -0x201, 0x3a, 0x3a, column)
                }
            };

        let width = get_drawable_width(ctx)?;
        let bounce =
            *BUTTON_PRESS_BOUNCE
                .get(press as i64 as usize)
                .ok_or(Fault::index_out_of_range(press as i64, 6))?;
        let half = bounce / 2;
        let x = width
            .wrapping_add(column.wrapping_mul(0x58))
            .wrapping_sub(half)
            .wrapping_add(shift);
        let lift = ctx
            .i32_at(AppContext::LETTERBOX_SHIFT)?
            .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
            .wrapping_add(half);
        let size = bounce.wrapping_add(grow);

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            ctx.mapicon_sheet
                .as_deref()
                .ok_or(Fault::null_pointer())?,
            x,
            top.wrapping_sub(lift),
            size,
            size,
            cut,
        );
    }

    let mut any = false;

    for powerup in 0..6 {
        if powerup_available(ctx, powerup)? {
            ctx.set_i32_at(AppContext::DRAW_TEMP_0, 1)?;
            any = powerup == 5;
        }
    }

    let any = any || ctx.i32_at(AppContext::DRAW_TEMP_0)? == 1;
    let touched =
        ctx.u8_at(AppContext::INPUT_BLOCKED)? != 0 || ctx.i32_at(AppContext::UI_TAP_LOCKOUT)? > 0;

    if touched && any {
        let offset = if ctx.i32_at(AppContext::TOOLTIP_ITEM)? == 5 {
            0x1b8
        } else {
            let mut columns = 5i32;
            let mut column = 5i32;

            loop {
                columns = columns.wrapping_sub(powerup_available(ctx, column)? as i32);

                if column == 0 {
                    break;
                }

                let next = column - 1;

                column = next;

                if next == ctx.i32_at(AppContext::TOOLTIP_ITEM)? {
                    break;
                }
            }

            columns.wrapping_mul(0x58)
        };

        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xd8);

        let left = get_drawable_width(ctx)?.wrapping_add(-0x2ce);

        fill_rect_f(
            draw_context(&mut ctx.draw)?,
            left as f32,
            134.0,
            710.0,
            159.0,
        );

        let tip = get_drawable_width(ctx)?
            .wrapping_add(offset)
            .wrapping_add(-0x1e4);

        ctx.set_i32_at(AppContext::POLYGON_XS, tip)?;

        let right = get_drawable_width(ctx)?
            .wrapping_add(offset)
            .wrapping_add(-0x1d5);

        ctx.set_i32_at(AppContext::POLYGON_XS + 4, right)?;

        let left = get_drawable_width(ctx)?
            .wrapping_add(offset)
            .wrapping_add(-0x1f3);

        ctx.set_i32_at(AppContext::POLYGON_XS + 8, left)?;
        ctx.set_i32_at(AppContext::POLYGON_YS, 0x76)?;
        ctx.set_i32_at(AppContext::POLYGON_YS + 4, 0x86)?;
        ctx.set_i32_at(AppContext::POLYGON_YS + 8, 0x86)?;

        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xd8);

        let xs = [
            ctx.i32_at(AppContext::POLYGON_XS)?,
            ctx.i32_at(AppContext::POLYGON_XS + 4)?,
            ctx.i32_at(AppContext::POLYGON_XS + 8)?,
        ];
        let ys = [
            ctx.i32_at(AppContext::POLYGON_YS)?,
            ctx.i32_at(AppContext::POLYGON_YS + 4)?,
            ctx.i32_at(AppContext::POLYGON_YS + 8)?,
        ];

        fill_polygon(draw_context(&mut ctx.draw)?, &xs, &ys, 3);
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

        let item = ctx.i32_at(AppContext::TOOLTIP_ITEM)?;

        if item == 0 && get_powerup_available(ctx)? == 0 {
            let x = get_drawable_width(ctx)?.wrapping_add(-0x16b);

            text_block_draw(&mut ctx.draw, &mut ctx.text_blocks, 0, x, 0x91, 1, 1.0)?;
        } else {
            for line in 0..4i32 {
                if line == 0 {
                    let label = ctx.i32_at(AppContext::TOOLTIP_PAGE)? as i64 as usize;

                    if item == 0 {
                        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0, 0xff, 0xff);

                        let x = get_drawable_width(ctx)?.wrapping_add(-0x16b);
                        let texture = ctx
                            .label_texts
                            .get(label)
                            .and_then(|slot| slot.as_ref())
                            .ok_or(Fault::null_pointer())?;

                        draw_surface_aligned(
                            draw_context(&mut ctx.draw)?,
                            Surface::Label(texture),
                            x,
                            0x91,
                            1,
                        );
                        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                    } else {
                        let x = get_drawable_width(ctx)?.wrapping_add(-0x16b);
                        let texture = ctx
                            .label_texts
                            .get(label)
                            .and_then(|slot| slot.as_ref())
                            .ok_or(Fault::null_pointer())?;

                        draw_surface_aligned(
                            draw_context(&mut ctx.draw)?,
                            Surface::Label(texture),
                            x,
                            0x91,
                            1,
                        );
                    }

                    continue;
                }

                let index = find_item_index(ctx, ctx.i32_at(AppContext::TOOLTIP_ITEM)?)?;
                let lines = get_item_description(ctx, index);
                let blank = lines
                    .get(line as usize - 1)
                    .ok_or(Fault::index_out_of_range(line as i64 - 1, lines.len() as i64))?
                    .as_slice()
                    == BLANK_LINE;

                if blank {
                    continue;
                }

                let label = (ctx.i32_at(AppContext::TOOLTIP_PAGE)? as i64).wrapping_add(line as i64)
                    as usize;
                let x = get_drawable_width(ctx)?.wrapping_add(-0x16b);
                let texture = ctx
                    .label_texts
                    .get(label)
                    .and_then(|slot| slot.as_ref())
                    .ok_or(Fault::null_pointer())?;

                draw_surface_aligned(
                    draw_context(&mut ctx.draw)?,
                    Surface::Label(texture),
                    x,
                    line.wrapping_mul(0x24).wrapping_add(0x91),
                    1,
                );
            }
        }
    }

    if get_battle_status(ctx)? != 0 || touch_is_down(ctx)? == 0 {
        ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;
    }

    let y = ctx
        .i32_at(AppContext::LETTERBOX_PAD)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    set_draw_origin(ctx, 0, y)
}
