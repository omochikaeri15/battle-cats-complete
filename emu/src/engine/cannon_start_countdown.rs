use crate::{Fault, ops};

use super::{AppContext, get_cat_combo_bonus};

pub fn cannon_start_countdown(ctx: &AppContext, recharge: i32) -> Result<i32, Fault> {
    let bonus = get_cat_combo_bonus(ctx, &ctx.combo_store, 3, -1)?;
    let frames =
        ops::div_100(100i32.wrapping_sub(bonus).wrapping_mul(recharge) as i64) as i32;

    Ok(if frames > 0 { frames } else { 0 })
}
