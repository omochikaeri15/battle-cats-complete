use crate::Fault;

use super::{get_talent_value, has_talent, max_i32, read_flag, AppContext};

pub fn stat_strengthen_threshold(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x233188) as usize);
    }

    let cell = ((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e608) as usize;

    let threshold = if ctx.i32_at(cell)? != 0 {
        let threshold = ctx.i32_at(cell)?;

        if !has_talent(ctx, faction, unit_id, form, 0xa)? {
            return Ok(max_i32(threshold, 0));
        }

        threshold
    } else {
        if !has_talent(ctx, faction, unit_id, form, 0xa)? {
            return Ok(max_i32(0, 0));
        }

        0x64
    };

    Ok(max_i32(threshold.wrapping_sub(get_talent_value(ctx, faction, unit_id, form, 0xa, 0)?), 0))
}
