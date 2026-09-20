use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, BUTTON_PRESS_BOUNCE, POPUP_GROW_TABLE, Surface, dialog_button_rect,
    draw_context, draw_cut_scaled, draw_nine_slice, draw_surface_scaled, fill_rect,
    get_design_height2, get_drawable_width, hit_test_rect, imgcut_get_sprite_cut, set_color,
    set_tint, set_tint_alpha, texture_get_height, texture_get_width,
};

const ONE_BUTTON: ([i32; 3], [i32; 3], usize) = ([0, 0, 0], [6, 0, 0], 1);
const TWO_BUTTONS: ([i32; 3], [i32; 3], usize) = ([0, 1, 0], [5, 4, 0], 2);
const THREE_BUTTONS: ([i32; 3], [i32; 3], usize) = ([0, 1, 2], [5, 4, 6], 3);

pub fn dialog_draw(ctx: &mut AppContext, dialog: u64, is_top: u8) -> Result<(), Fault> {
    let sheet = Rc::clone(ctx.dialog_sheet.as_ref().ok_or(Fault::null_pointer())?);
    let this = ctx
        .dialogs
        .objects
        .get(&dialog)
        .cloned()
        .ok_or(Fault::null_pointer())?;

    if this.flags & 4 != 0 {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xb2);

        let shift = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, shift, width, height);
        set_tint_alpha(draw_context(&mut ctx.draw)?, 0xff);
    }

    let grow = *POPUP_GROW_TABLE
        .get(this.frame as i64 as usize)
        .ok_or(Fault::index_out_of_range(this.frame as i64, POPUP_GROW_TABLE.len() as i64))?;

    if grow == 0 {
        return Ok(());
    }

    let half_w = ops::div_2(this.width);
    let x = this
        .x
        .wrapping_add(half_w)
        .wrapping_add(ops::div_neg_100(half_w.wrapping_mul(grow) as i64) as i32);
    let half_h = ops::div_2(this.height);
    let y = this
        .y
        .wrapping_add(half_h)
        .wrapping_add(ops::div_neg_100(half_h.wrapping_mul(grow) as i64) as i32);
    let width = ops::div_100(this.width.wrapping_mul(grow) as i64) as i32;
    let height = ops::div_100(this.height.wrapping_mul(grow) as i64) as i32;
    let border_x = imgcut_get_sprite_cut(&sheet, 8)?[0];
    let border_y = imgcut_get_sprite_cut(&sheet, 8)?[1];

    draw_nine_slice(
        draw_context(&mut ctx.draw)?,
        &sheet,
        x,
        y,
        width,
        height,
        2.7,
        0,
        border_x,
        border_y,
        imgcut_get_sprite_cut(&sheet, 8)?[2],
        imgcut_get_sprite_cut(&sheet, 8)?[3],
    );

    if this.state.wrapping_sub(1) as u32 > 1 {
        return Ok(());
    }

    for line in this.lines.iter() {
        for segment in line.segments.iter() {
            let color = if segment.flash == 0 {
                segment.color
            } else {
                let tick = this.tick;
                let phase = tick.wrapping_sub(
                    (if tick < 0 { tick.wrapping_add(3) } else { tick }) & -4,
                );

                if phase > 1 { [0xff, 0, 0xff] } else { [0xff, 0xff, 0] }
            };

            set_tint(draw_context(&mut ctx.draw)?, color[0], color[1], color[2], 0xff);

            let left = ops::div_2(this.width)
                .wrapping_add(this.x)
                .wrapping_sub(ops::div_2(line.width));
            let across = ops::cvttss2si(segment.x as f32 * line.scale + left as f32);
            let surface = Surface::Label(&segment.texture);
            let down = ops::div_2(this.height)
                .wrapping_add(this.y)
                .wrapping_sub(ops::div_2(texture_get_height(surface)))
                .wrapping_add(line.y)
                .wrapping_add((this.lines.len() as u32).wrapping_mul(0xffffffee) as i32)
                .wrapping_add(0x12);
            let wide = ops::cvttss2si(texture_get_width(surface) as f32 * line.scale);
            let tall = texture_get_height(surface);

            draw_surface_scaled(draw_context(&mut ctx.draw)?, surface, across, down, wide, tall);
        }
    }

    let (ids, cuts, count) = match this.kind {
        1 => ONE_BUTTON,
        2 => TWO_BUTTONS,
        3 => THREE_BUTTONS,
        _ => ([0; 3], [0; 3], 0),
    };

    for slot in 0..count {
        if this.visible[slot] == 0 {
            continue;
        }

        let mut bounce = 0i32;

        if ids[slot] == this.button {
            bounce = *BUTTON_PRESS_BOUNCE.get(this.timer as i64 as usize).ok_or(
                Fault::index_out_of_range(this.timer as i64, BUTTON_PRESS_BOUNCE.len() as i64),
            )?;
        }

        let rect = dialog_button_rect(&this, ids[slot]);
        let dim = if this.lit[slot] == 0 {
            this.dimmable[slot] != 0 || this.greyed[slot] != 0
        } else {
            this.greyed[slot] != 0
        };

        if dim {
            set_color(draw_context(&mut ctx.draw)?, 0x7f, 0x7f, 0x7f, 0xff);
        }

        let border_x =
            imgcut_get_sprite_cut(&sheet, 7)?[0].wrapping_sub(imgcut_get_sprite_cut(&sheet, 0)?[0]);
        let border_y =
            imgcut_get_sprite_cut(&sheet, 7)?[1].wrapping_sub(imgcut_get_sprite_cut(&sheet, 0)?[1]);
        let across = rect[0].wrapping_sub(ops::div_2(bounce));
        let down = rect[1].wrapping_sub(ops::div_2(bounce));
        let wide = bounce.wrapping_add(rect[2]);
        let tall = bounce.wrapping_add(rect[3]);

        draw_nine_slice(
            draw_context(&mut ctx.draw)?,
            &sheet,
            across,
            down,
            wide,
            tall,
            1.0,
            1,
            border_x,
            border_y,
            imgcut_get_sprite_cut(&sheet, 7)?[2],
            imgcut_get_sprite_cut(&sheet, 7)?[3],
        );

        if this.flags & 0x10 == 0 {
            let cut = cuts[slot];
            let cut_w = imgcut_get_sprite_cut(&sheet, cut)?[2];
            let cut_h = imgcut_get_sprite_cut(&sheet, cut)?[3];

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                &sheet,
                ops::div_2(rect[2])
                    .wrapping_add(across)
                    .wrapping_sub(ops::div_2(cut_w)),
                ops::div_2(rect[3])
                    .wrapping_add(down)
                    .wrapping_sub(ops::div_2(cut_h)),
                imgcut_get_sprite_cut(&sheet, cut)?[2].wrapping_add(bounce),
                bounce.wrapping_add(imgcut_get_sprite_cut(&sheet, cut)?[3]),
                cut,
            );
        }

        let highlighted = if count == 1 && this.flags & 0x10 == 0 {
            is_top != 0
        } else if ctx.u8_at(AppContext::TOUCH_DOWN_LATCH)? == 0 {
            false
        } else {
            (hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])? as u8) & is_top != 0
        };

        if highlighted {
            let half = ops::div_2(this.tick);
            let blink = half.wrapping_sub(ops::div_2(half).wrapping_mul(2));
            let border_x = imgcut_get_sprite_cut(&sheet, 9)?[0]
                .wrapping_sub(imgcut_get_sprite_cut(&sheet, 2)?[0]);
            let border_y = imgcut_get_sprite_cut(&sheet, 9)?[1]
                .wrapping_sub(imgcut_get_sprite_cut(&sheet, 2)?[1]);

            draw_nine_slice(
                draw_context(&mut ctx.draw)?,
                &sheet,
                across.wrapping_sub(1),
                down.wrapping_sub(1),
                wide.wrapping_add(2),
                tall.wrapping_add(2),
                1.5,
                blink.wrapping_add(2),
                border_x,
                border_y,
                imgcut_get_sprite_cut(&sheet, 9)?[2],
                imgcut_get_sprite_cut(&sheet, 9)?[3],
            );
        }

        let restore = if this.lit[slot] == 0 {
            this.dimmable[slot] != 0 || this.greyed[slot] != 0
        } else {
            this.greyed[slot] != 0
        };

        if restore {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
        }
    }

    if this.flags & 8 != 0 {
        let mut bounce = 0i32;

        if this.button == -2 {
            bounce = *BUTTON_PRESS_BOUNCE.get(this.timer as i64 as usize).ok_or(
                Fault::index_out_of_range(this.timer as i64, BUTTON_PRESS_BOUNCE.len() as i64),
            )?;
        }

        let rect = [
            this.x.wrapping_add(this.width).wrapping_add(-0x24),
            this.y.wrapping_add(-0x24),
            0x48,
            0x48,
        ];
        let across = rect[0].wrapping_sub(ops::div_2(bounce));
        let down = rect[1].wrapping_sub(ops::div_2(bounce));
        let wide = bounce.wrapping_add(rect[2]);
        let tall = bounce.wrapping_add(rect[3]);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, &sheet, across, down, wide, tall, 0xb);

        if ctx.u8_at(AppContext::TOUCH_DOWN_LATCH)? != 0
            && is_top != 0
            && hit_test_rect(ctx, rect[0], rect[1], rect[2], rect[3])?
        {
            let half = ops::div_2(this.tick);
            let blink = half.wrapping_sub(ops::div_2(half).wrapping_mul(2));

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                &sheet,
                across.wrapping_sub(1),
                down.wrapping_sub(1),
                wide.wrapping_add(2),
                tall.wrapping_add(2),
                blink.wrapping_add(0xc),
            );
        }
    }

    if let Some(paint) = this.on_draw {
        draw_context(&mut ctx.draw)?;
        paint(ctx)?;
    }

    Ok(())
}
