use crate::Fault;

use super::{get_talent_value, has_talent, max_i32, read_flag, AppContext};

pub fn stat_weaken_to(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x233184) as usize);
    }

    let row = (unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8;

    let mut weaken_to = if ctx.i32_at((row + 0x9e5fc) as usize)? != 0 { ctx.i32_at((row + 0x9e604) as usize)? } else { 0x64 };

    if has_talent(ctx, faction, unit_id, form, 0x1)? {
        weaken_to = weaken_to.wrapping_sub(get_talent_value(ctx, faction, unit_id, form, 0x1, 2)?);
    }

    Ok(max_i32(weaken_to, 0))
}
