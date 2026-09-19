use crate::Fault;

use super::{AppContext, get_button_unit_row, read_flag, stat_conjure_unit_id};

pub fn slot_conjure_ready(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    let unit_id = get_button_unit_row(ctx, faction, slot)?.wrapping_add(-2);
    let mut form = 0i32;

    if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
            let row = get_button_unit_row(ctx, 1, slot)? as i64;

            form = ctx.i32_at((row * 4 + AppContext::FACTION_1_UNIT_FORMS as i64) as usize)?;
        }
    } else if faction == 0 && slot as u32 <= 9 {
        form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (slot as u32 as usize) * 4)?;
    }

    if stat_conjure_unit_id(ctx, 0, unit_id, form)? < 0 {
        return Ok(false);
    }

    let ready = AppContext::faction_flags(faction)
        .wrapping_add(((slot as i64) * 4) as usize)
        .wrapping_add(AppContext::WALLET_CONJURE_READY);

    Ok(ctx.i32_at(ready)? == 1)
}
