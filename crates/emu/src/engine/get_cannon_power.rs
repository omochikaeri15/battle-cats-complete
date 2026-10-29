use crate::{Fault, ops};

use super::{AppContext, get_base_upgrade, get_cat_combo_bonus, get_treasure_value};

pub fn get_cannon_power(ctx: &mut AppContext) -> Result<i32, Fault> {
    let upgrade = get_base_upgrade(ctx, 0)?.wrapping_mul(0x32);
    let base = upgrade
        .wrapping_add(get_treasure_value(ctx, &ctx.treasure_store, 2)?)
        .wrapping_add(100);
    let bonus = get_cat_combo_bonus(ctx, &ctx.combo_store, 6, -1)?;
    let power = ops::div_100(bonus.wrapping_add(100).wrapping_mul(base) as i64) as i32;

    Ok(if power <= 0 { 0 } else { power })
}
