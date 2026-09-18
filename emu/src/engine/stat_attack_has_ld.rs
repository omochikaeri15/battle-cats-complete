use crate::Fault;

use super::{read_flag, AppContext, STAT_ATTACK_COLUMNS};

pub fn stat_attack_has_ld(ctx: &AppContext, faction: i32, unit_id: i32, form: i32, attack: i32) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        let column = *STAT_ATTACK_COLUMNS.get((18 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_has_ld",
            index: attack as i64,
            limit: 3,
        })?;

        AppContext::cat_stat(unit_id, form, ((column as i64) * 4) as usize)
    } else {
        let column = *STAT_ATTACK_COLUMNS.get((21 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_has_ld",
            index: attack as i64,
            limit: 3,
        })?;

        AppContext::enemy_stat(unit_id, ((column as i64) * 4) as usize)
    };

    Ok(ctx.i32_at(cell)? != 0)
}
