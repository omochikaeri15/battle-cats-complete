use crate::Fault;

use super::{get_button_unit_row, read_flag, AppContext};

pub fn get_button_unit_form(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(0);
        }

        let row = get_button_unit_row(ctx, 1, slot)?;

        return ctx.i32_at(((row as i64) * 4 + AppContext::FACTION_1_UNIT_FORMS as i64) as usize);
    }

    if faction != 0 {
        return Ok(0);
    }

    if slot as u32 > 9 {
        return Ok(0);
    }

    ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (slot as u32 as usize) * 4)
}
