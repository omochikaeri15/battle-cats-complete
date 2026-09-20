use crate::{Fault, operation};

use super::{
    AppContext, Surface, draw_context, draw_cut, draw_surface_aligned, fill_rect,
    get_drawable_width, glow_set, set_tint,
};

pub fn combo_banner_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.i32_at(AppContext::COMBO_BANNER_PHASE)? <= 0 {
        return Ok(());
    }

    let third = ctx.combo_banner_texts[2].is_some();

    glow_set(draw_context(&mut ctx.draw)?, 2);
    set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x4d, 0xff);

    let step = ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)?;
    let sub = ctx.i32_at(AppContext::COMBO_BANNER_SUB)?;
    let lead = operation::idiv(0x3c, step)
        .ok_or(Fault::divide(step as i64))?
        .wrapping_mul(sub);
    let width = get_drawable_width(ctx)?;
    let grow = operation::idiv(0x78, step)
        .ok_or(Fault::divide(step as i64))?
        .wrapping_mul(sub);
    let (top, height): (i32, i32) = if third {
        (-0xf, grow.wrapping_add(0x1e))
    } else {
        (0, grow)
    };
    let y = top.wrapping_sub(lead).wrapping_add(0xa0);

    fill_rect(draw_context(&mut ctx.draw)?, 0, y, width, height);
    glow_set(draw_context(&mut ctx.draw)?, 0);

    if ctx.i32_at(AppContext::COMBO_BANNER_SUB)? < ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)? {
        return Ok(());
    }

    let sheet = ctx.img004_sheet.clone();
    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
    let x = get_drawable_width(ctx)?.wrapping_add(-0x57);

    draw_cut(
        draw_context(&mut ctx.draw)?,
        sheet,
        x,
        0xc2i32.wrapping_sub(top),
        8,
    );
    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    if let Some(text) = ctx.combo_banner_texts[0] {
        let x = ctx
            .i32_at(AppContext::COMBO_BANNER_X)?
            .wrapping_sub(operation::div_2(get_drawable_width(ctx)?));

        draw_surface_aligned(
            draw_context(&mut ctx.draw)?,
            Surface::Label(&text),
            x,
            top.wrapping_add(0x6e),
            1,
        );
    }

    let ticks = ctx.i32_at(AppContext::COMBO_BANNER_TICKS)?;
    let phase = ticks.wrapping_sub(operation::div_4(ticks) * 4);

    if (phase as u32) <= 1 {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);
    } else {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0, 0xff, 0xff);
    }

    if let Some(text) = ctx.combo_banner_texts[1] {
        let x = ctx
            .i32_at(AppContext::COMBO_BANNER_X)?
            .wrapping_sub(operation::div_2(get_drawable_width(ctx)?));

        draw_surface_aligned(
            draw_context(&mut ctx.draw)?,
            Surface::Label(&text),
            x,
            top.wrapping_add(0xaa),
            1,
        );
    }

    if let Some(text) = ctx.combo_banner_texts[2] {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

        let x = ctx
            .i32_at(AppContext::COMBO_BANNER_X)?
            .wrapping_sub(operation::div_2(get_drawable_width(ctx)?));

        draw_surface_aligned(
            draw_context(&mut ctx.draw)?,
            Surface::Label(&text),
            x,
            top.wrapping_add(0xc8),
            1,
        );
    }

    Ok(())
}
