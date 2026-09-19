use crate::Fault;

use super::{get_tech_level, has_fixed_lineup, AppContext};

pub fn get_base_upgrade(ctx: &mut AppContext, upgrade: i32) -> Result<i32, Fault> {
    let mut key = upgrade;

    if has_fixed_lineup(ctx, -1, -1, -1)? {
        key = key.wrapping_sub((key > 0) as i32);

        if ctx.fixed_lineup_store.ability_levels.contains_key(&key) {
            return Ok(*ctx.fixed_lineup_store.ability_levels.entry(key).or_insert(0));
        }
    }

    get_tech_level(ctx, (((key as i64) << 3) + AppContext::TECH_LEVELS as i64) as usize)
}
