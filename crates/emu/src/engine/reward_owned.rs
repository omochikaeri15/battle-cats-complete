use crate::Fault;

use super::{AppContext, reward_unit_id};

pub fn reward_owned(ctx: &AppContext, id: i32) -> Result<bool, Fault> {
    if id == 0x3ea {
        return Ok(ctx.i32_at(AppContext::REWARD_1002_OWNED)? != 0);
    }

    if (id.wrapping_sub(0x2710) as u32) <= 0x270f {
        let unit = reward_unit_id(ctx, id)?;

        if unit == -1 {
            return Ok(false);
        }

        return Ok(ctx
            .i32_at(((unit as i64) * 4 + AppContext::REWARD_UNITS_OWNED as i64) as usize)?
            != 0);
    }

    if id >= 0x4e20 {
        let unit = reward_unit_id(ctx, id)?;

        if unit == -1 {
            return Ok(false);
        }

        return Ok(ctx
            .i32_at(((unit as i64) * 4 + AppContext::REWARD_FORMS_OWNED as i64) as usize)?
            != 0);
    }

    let count = ctx.drop_chara_rows.len() as i32;

    if count <= 0 {
        return Ok(false);
    }

    for row in ctx.drop_chara_rows.iter().take(count as u32 as usize) {
        if *row.first().ok_or(Fault::index_out_of_range(0, 0))? == id
        {
            let slot = *row.get(1).ok_or(Fault::index_out_of_range(1, row.len() as i64))?;

            if slot == -1 {
                return Ok(false);
            }

            return Ok(ctx
                .i32_at(((slot as i64) * 4 + AppContext::EVENT_UNIT_OWNED as i64) as usize)?
                != 0);
        }
    }

    Ok(false)
}
