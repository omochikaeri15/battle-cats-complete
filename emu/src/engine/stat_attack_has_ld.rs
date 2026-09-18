use crate::Fault;

use super::{read_flag, AppContext, STAT_ATTACK_COLUMNS};

pub fn stat_attack_has_ld(ctx: &AppContext, team: i32, unit_id: i32, form: i32, attack: i32) -> Result<bool, Fault> {
    let cell = if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        let column = *STAT_ATTACK_COLUMNS.get((18 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_has_ld",
            index: attack as i64,
            limit: 3,
        })?;

        (unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + (column as i64) * 4 + 0x9e568
    } else {
        let column = *STAT_ATTACK_COLUMNS.get((21 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_has_ld",
            index: attack as i64,
            limit: 3,
        })?;

        (unit_id.wrapping_add(2) as i64) * 0x1c4 + (column as i64) * 4 + 0x233108
    };

    Ok(ctx.i32_at(cell as usize)? != 0)
}
