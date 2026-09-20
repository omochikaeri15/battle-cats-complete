use crate::{Fault, operation};

use super::{
    AppContext, draw_context, draw_cut_scaled, draw_number_plain, get_drawable_width,
    get_max_money, get_money, get_right_inset_logical, set_draw_origin,
};

pub fn draw_money(ctx: &mut AppContext) -> Result<(), Fault> {
    let inset = get_right_inset_logical(ctx)?.wrapping_neg();
    let top = ctx
        .i32_at(AppContext::LETTERBOX_PAD)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    set_draw_origin(ctx, inset, top)?;

    let phase = ctx.i32_at(AppContext::WORKER_UPGRADE_VFX)?;
    let folded = if phase >= 0 {
        phase
    } else {
        phase.wrapping_add(3)
    };
    let folded = (!folded | 3).wrapping_add(phase);
    let base = if (folded as u32) < 2 { 0x44 } else { 0 };

    ctx.set_i32_at(AppContext::DRAW_TEMP_3, base)?;

    let right = get_drawable_width(ctx)?.wrapping_add(-4);

    ctx.set_i32_at(AppContext::DRAW_TEMP_1, right)?;

    let x = right as f32;
    let base = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
    let y = 0xai32.wrapping_sub(
        ctx.i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?),
    ) as f32;
    let cap = operation::div_100(get_max_money(ctx, AppContext::faction_flags(0))?);
    let sheet = ctx.img001_sheet.clone();
    let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
    let bounds = draw_number_plain(
        draw_context(&mut ctx.draw)?,
        digits,
        base,
        cap,
        0,
        x,
        y,
        -1.0,
        0x1e,
        2,
        0,
    )?;
    let x = bounds.left;
    let y = bounds.top;
    let cut = ctx.i32_at(AppContext::DRAW_TEMP_3)?.wrapping_add(0xb);

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        digits,
        operation::cvttss2si(bounds.right),
        operation::cvttss2si(y),
        0x1e,
        0x2a,
        cut,
    );

    let cut = ctx.i32_at(AppContext::DRAW_TEMP_3)?.wrapping_add(0xa);

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        digits,
        operation::cvttss2si(x + -29.0),
        operation::cvttss2si(y),
        0x1e,
        0x2a,
        cut,
    );

    let x = x + -28.0;
    let base = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
    let money = operation::div_100(get_money(ctx, AppContext::faction_flags(0))?);

    draw_number_plain(
        draw_context(&mut ctx.draw)?,
        digits,
        base,
        money,
        0,
        x,
        y,
        -1.0,
        0,
        2,
        0,
    )?;

    let top = ctx
        .i32_at(AppContext::LETTERBOX_PAD)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    set_draw_origin(ctx, 0, top)
}
