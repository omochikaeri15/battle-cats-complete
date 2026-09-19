use crate::{operation, Fault};

use super::{draw_context, fill_rect, get_deck_cooldown, get_deck_cooldown_max, get_drawable_width, get_setting, get_unit_recharge, set_tint, AppContext, DECK_SLOT_X_TABLE};

const SITE: &str = "draw_cooldown_bar";

pub fn draw_cooldown_bar(ctx: &mut AppContext, slot: i32) -> Result<(), Fault> {
    let mut shift = 0i32;
    let recharge = get_unit_recharge(ctx, 0, slot)?;

    ctx.set_i32_at(AppContext::DRAW_TEMP_2, recharge)?;

    let wallet = AppContext::faction_flags(0);
    let elapsed = recharge.wrapping_sub(get_deck_cooldown(ctx, wallet, slot)?).wrapping_mul(0x5d);
    let span = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
    let filled = operation::idiv(elapsed, span).ok_or(Fault::divide(SITE, span as i64))?;
    let filled = if filled < 0x5d { filled } else { 0x5d };

    ctx.set_i32_at(AppContext::DRAW_TEMP_3, filled)?;

    let remaining = span.wrapping_sub(get_deck_cooldown_max(ctx, wallet, slot)?).wrapping_mul(0x5d);
    let span = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
    let excess = operation::idiv(remaining, span).ok_or(Fault::divide(SITE, span as i64))?;

    ctx.set_i32_at(AppContext::DRAW_TEMP_4, 0x61)?;
    ctx.set_i32_at(AppContext::DRAW_TEMP_5, 0xe)?;

    let filled = ctx.i32_at(AppContext::DRAW_TEMP_3)?;

    ctx.set_i32_at(AppContext::DRAW_TEMP_6, filled)?;
    ctx.set_i32_at(AppContext::DRAW_TEMP_7, 0xa)?;

    let two_lines = ctx.u8_at(AppContext::DECK_TWO_LINES)?;
    let mut column = slot.wrapping_sub(operation::div_5(slot as u32 as i64) as i32 * 5);

    if two_lines == 0 {
        column = slot;
    }

    if slot < 5 {
        column = slot;

        if two_lines != 0 {
            shift = 0i32.wrapping_sub(get_setting(&ctx.settings, b"battle_slot_2lines_line", 0x5a)?);
        }
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let seat = *DECK_SLOT_X_TABLE
        .get(column as i64 as usize)
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: column as i64, limit: DECK_SLOT_X_TABLE.len() as i64 })?;
    let x = operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + seat.wrapping_add(6) as f64);
    let y = ctx
        .i32_at(AppContext::LETTERBOX_SHIFT)?
        .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(shift))
        .wrapping_add(0x25e);

    fill_rect(draw_context(&mut ctx.draw)?, x, y, 0x61, 0xe);

    if ctx.i32_at(wallet.wrapping_add(AppContext::WALLET_RED_GAUGE_FRAMES).wrapping_add(((slot as i64) * 4) as usize))? > 0 {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0, 0, 0xff);

        let x = operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + seat.wrapping_add(8) as f64);

        shift = shift.wrapping_add(0x260);

        let y = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(shift).wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        fill_rect(draw_context(&mut ctx.draw)?, x, y, excess, 0xa);
    } else {
        shift = shift.wrapping_add(0x260);
    }

    set_tint(draw_context(&mut ctx.draw)?, 0, 0xff, 0xff, 0xff);

    let x = operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + seat.wrapping_add(8) as f64);
    let y = shift
        .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let filled = ctx.i32_at(AppContext::DRAW_TEMP_3)?;

    fill_rect(draw_context(&mut ctx.draw)?, x, y, filled, 0xa);

    Ok(())
}
