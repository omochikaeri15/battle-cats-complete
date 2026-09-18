use crate::Fault;

use super::{get_cat_combo_bonus, get_talent_value, read_flag, AppContext};

pub fn stat_critical_chance(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x23316c) as usize);
    }

    let base = ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e5e4) as usize)?;
    let chance = get_talent_value(ctx, faction, unit_id, form, 0xd, 0)?.wrapping_add(base);

    if chance <= 0 {
        return Ok(chance);
    }

    Ok(chance.wrapping_add(get_cat_combo_bonus(ctx, &ctx.combo_store, 0x18, unit_id)?))
}
