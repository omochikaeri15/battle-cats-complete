use std::rc::Rc;

use crate::Fault;

use super::{get_button_unit_row, read_flag, AppContext, SheetTable};

pub fn get_sheet_table(ctx: &AppContext, faction: i32, button: i32) -> Result<Option<SheetTable>, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        if read_flag(ctx, AppContext::faction_flags(faction))? & 2 == 0 {
            return Ok(None);
        }

        return Ok(Some(Rc::clone(&ctx.enemy_sheets)));
    }

    let form = if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(Some(Rc::clone(&ctx.unit_sheets[0])));
        }

        let row = get_button_unit_row(ctx, 1, button)?;

        ctx.i32_at(AppContext::FACTION_1_UNIT_FORMS.wrapping_add((row as i64 as usize).wrapping_mul(4)))?
    } else if faction == 0 && button as u32 <= 9 {
        ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as usize) * 4)?
    } else {
        return Ok(Some(Rc::clone(&ctx.unit_sheets[0])));
    };

    if form == 0 {
        return Ok(Some(Rc::clone(&ctx.unit_sheets[0])));
    }

    let form = if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(None);
        }

        let row = get_button_unit_row(ctx, 1, button)?;

        ctx.i32_at(AppContext::FACTION_1_UNIT_FORMS.wrapping_add((row as i64 as usize).wrapping_mul(4)))?
    } else if faction == 0 && button as u32 <= 9 {
        ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as usize) * 4)?
    } else {
        return Ok(None);
    };

    if form == 1 {
        return Ok(Some(Rc::clone(&ctx.unit_sheets[1])));
    }

    let form = if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(None);
        }

        let row = get_button_unit_row(ctx, 1, button)?;

        ctx.i32_at(AppContext::FACTION_1_UNIT_FORMS.wrapping_add((row as i64 as usize).wrapping_mul(4)))?
    } else if faction == 0 && button as u32 <= 9 {
        ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as usize) * 4)?
    } else {
        return Ok(None);
    };

    if form == 2 {
        return Ok(Some(Rc::clone(&ctx.unit_sheets[2])));
    }

    let form = if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(None);
        }

        let row = get_button_unit_row(ctx, 1, button)?;

        ctx.i32_at(AppContext::FACTION_1_UNIT_FORMS.wrapping_add((row as i64 as usize).wrapping_mul(4)))?
    } else if faction == 0 && button as u32 <= 9 {
        ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as usize) * 4)?
    } else {
        return Ok(None);
    };

    Ok(if form == 3 { Some(Rc::clone(&ctx.unit_sheets[3])) } else { None })
}
