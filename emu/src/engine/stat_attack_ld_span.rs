use crate::Fault;

use super::{AppContext, STAT_ATTACK_COLUMNS, read_flag};

pub fn stat_attack_ld_span(
    ctx: &AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    attack: i32,
) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        let column = *STAT_ATTACK_COLUMNS
            .get((30 + attack as i64) as usize)
            .ok_or(Fault::index_out_of_range(attack as i64, 3))?;

        AppContext::cat_stat(unit_id, form, ((column as i64) * 4) as usize)
    } else {
        let column = *STAT_ATTACK_COLUMNS
            .get((33 + attack as i64) as usize)
            .ok_or(Fault::index_out_of_range(attack as i64, 3))?;

        AppContext::enemy_stat(unit_id, ((column as i64) * 4) as usize)
    };

    let value = ctx.i32_at(cell)?;
    let shift = (attack != 0) as u32 * 2;

    Ok(value << shift)
}
