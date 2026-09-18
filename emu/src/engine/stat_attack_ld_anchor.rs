use crate::Fault;

use super::{read_flag, AppContext, STAT_ATTACK_COLUMNS};

pub fn stat_attack_ld_anchor(ctx: &AppContext, team: i32, unit_id: i32, form: i32, attack: i32) -> Result<i32, Fault> {
    let cell = if read_flag(ctx, (team as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        let column = *STAT_ATTACK_COLUMNS.get((24 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_ld_anchor",
            index: attack as i64,
            limit: 3,
        })?;

        (unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + (column as i64) * 4 + 0x9e568
    } else {
        let column = *STAT_ATTACK_COLUMNS.get((27 + attack as i64) as usize).ok_or(Fault::IndexOutOfRange {
            site: "stat_attack_ld_anchor",
            index: attack as i64,
            limit: 3,
        })?;

        (unit_id.wrapping_add(2) as i64) * 0x1c4 + (column as i64) * 4 + 0x233108
    };

    let value = ctx.i32_at(cell as usize)?;
    let shift = (attack != 0) as u32 * 2;

    Ok(value << shift)
}
