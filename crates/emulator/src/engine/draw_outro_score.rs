use crate::{Fault, ops};

use super::{
    button_bank_find, draw_context, draw_cut, draw_number_plain, fill_rect, get_drawable_width, get_stage_score, glow_set, imgcut_get_sprite_cut,
    is_score_stage, new_button_draw, set_tint, AppContext, OUTRO_SLIDE_TABLE,
};

pub fn draw_outro_score(ctx: &mut AppContext) -> Result<(), Fault> {
    let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

    if phase > 0 {
        let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
        let step = if (frame as u32) >= 0xc || phase != 1 { 0xc } else { frame };
        let banner = ctx.img004_sheet.clone();
        let banner = banner.as_deref().ok_or(Fault::null_pointer())?;
        let slide = *OUTRO_SLIDE_TABLE
            .get(step as i64 as usize)
            .ok_or(Fault::index_out_of_range(step as i64, OUTRO_SLIDE_TABLE.len() as i64))?;
        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_sub(slide).wrapping_add(-0xef);

        draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0xbe, 0xc);

        let phase = ctx.i32_at(AppContext::OUTRO_PHASE)?;

        if phase >= 2 {
            let frame = ctx.i32_at(AppContext::OUTRO_FRAME)?;
            let step = if (frame as u32) >= 0xc || phase != 2 { 0xc } else { frame };

            glow_set(draw_context(&mut ctx.draw)?, 2);
            set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

            let slide = *OUTRO_SLIDE_TABLE
                .get(step as i64 as usize)
                .ok_or(Fault::index_out_of_range(step as i64, OUTRO_SLIDE_TABLE.len() as i64))?;
            let left = 0i32.wrapping_sub(slide);
            let width = get_drawable_width(ctx)?;

            fill_rect(draw_context(&mut ctx.draw)?, left, 0x13b, width, 0x37);
            glow_set(draw_context(&mut ctx.draw)?, 0);

            if is_score_stage(ctx.event_items.as_ref()) {
                let digits = ctx.img001_sheet.clone();
                let digits = digits.as_deref().ok_or(Fault::null_pointer())?;
                let origin = ops::div_2(get_drawable_width(ctx)?).wrapping_sub(slide) as f32;
                let score = get_stage_score(ctx.event_items.as_ref().ok_or(Fault::null_pointer())?);
                let offset = imgcut_get_sprite_cut(banner, 5)?[2].wrapping_add(0x14);
                let lead = imgcut_get_sprite_cut(banner, 4)?[2].wrapping_add(0x14);
                let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0, score, offset, origin, 321.0, -1.0, lead, 1, 0)?;
                let cut = imgcut_get_sprite_cut(banner, 0xb)?[2];
                let x = ops::cvttss2si(bounds.left - cut as f32 + -20.0);

                draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x144, 0xb);

                let x = ops::cvttss2si(20.0 + bounds.right);

                draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x143, 4);

                if ctx.i32_at(AppContext::OUTRO_PHASE)? != 2 && ctx.i32_at(AppContext::NEW_BEST_SCORE)? == 1 {
                    let middle = (bounds.right - bounds.left) * 0.5 + bounds.left;
                    let cut = imgcut_get_sprite_cut(banner, 6)?[2];
                    let x = ops::cvttss2si(middle - ops::div_2(cut) as f32);
                    let ticks = ctx.i32_at(AppContext::OUTRO_TICKS)?;
                    let beat = ticks.wrapping_sub(ops::div_4(ticks) * 4);
                    let blink = (ops::div_2(beat as i8 as i32) as i8).wrapping_add(6) as u8;

                    draw_cut(draw_context(&mut ctx.draw)?, banner, x, 0x12c, blink as i32);
                }
            }
        }
    }

    if ctx.i32_at(AppContext::OUTRO_PHASE)? != 7 || ctx.u8_at(AppContext::CURTAIN_ACTIVE)? != 0 {
        return Ok(());
    }

    let ok = button_bank_find(&ctx.buttons, 0xc8).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, ok, 0, 0)?;

    let share = button_bank_find(&ctx.buttons, 0xc9).ok_or(Fault::null_pointer())?;

    new_button_draw(ctx, share, 0, 0)
}
