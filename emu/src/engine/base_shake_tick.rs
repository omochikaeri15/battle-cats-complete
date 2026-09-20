use crate::{Fault, ops};

use super::{AppContext, call_rng, get_battle_status, sin_deg, std_map_int_shake_record_subscript};

pub fn base_shake_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? != 0
        || get_battle_status(ctx)? != 0
        || ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? != 0
    {
        ctx.base_shake.id = -1;
        ctx.base_shake.frame = 0;
        ctx.base_shake.unknown_2 = 0;
        ctx.base_shake.offset = 0;

        return Ok(());
    }

    for record in ctx.base_shake.records.values_mut() {
        if record.since < record.reset_frame {
            record.since = record.since.wrapping_add(1);
        }
    }

    let id = ctx.base_shake.id;

    if id == -1 {
        return Ok(());
    }

    let frame = ctx.base_shake.frame.wrapping_add(1);

    ctx.base_shake.frame = frame;

    if frame >= ctx.base_shake.records.entry(id).or_default().duration {
        ctx.base_shake.id = -1;
        ctx.base_shake.frame = 0;
        ctx.base_shake.unknown_2 = 0;
        ctx.base_shake.offset = 0;

        return Ok(());
    }

    let from = std_map_int_shake_record_subscript(&mut ctx.base_shake.records, &id).amplitude_from;
    let to = std_map_int_shake_record_subscript(&mut ctx.base_shake.records, &id).amplitude_to;
    let swing = to
        .wrapping_sub(
            std_map_int_shake_record_subscript(&mut ctx.base_shake.records, &id).amplitude_from,
        )
        .wrapping_mul(ctx.base_shake.frame);
    let duration = std_map_int_shake_record_subscript(&mut ctx.base_shake.records, &id).duration;
    let amplitude = ops::idiv(swing, duration)
        .ok_or(Fault::divide(duration as i64))?
        .wrapping_add(from);
    let roll = call_rng(ctx, 0x168);

    ctx.base_shake.offset = ops::cvttss2si(sin_deg(roll as f32) * amplitude as f32);

    Ok(())
}
