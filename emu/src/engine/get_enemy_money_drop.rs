use crate::{Fault, operation};

use super::{
    AppContext, ENEMY_STATS, ENEMY_STATS_STRIDE, EnemyStats, Entity, get_base_upgrade,
    get_cat_combo_bonus, get_treasure_value,
};

pub fn get_enemy_money_drop(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    bonus: i32,
) -> Result<i32, Fault> {
    let occupant = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? as i64;
    let drop = ctx.i32_at(
        (occupant * ENEMY_STATS_STRIDE as i64 + (ENEMY_STATS + EnemyStats::CASH_DROP) as i64)
            as usize,
    )?;
    let accounting = get_base_upgrade(ctx, 8)?;
    let treasure = get_treasure_value(ctx, &ctx.treasure_store, 5)?;
    let mut percent = 0x64i32;

    if ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::DOUBLE_BOUNTY_STATE,
    ))? == 2
    {
        ctx.set_i32_at(
            AppContext::entity_field(faction, slot, Entity::DOUBLE_BOUNTY_STATE),
            0,
        )?;
        percent = 0xc8;
    }

    let scaled = accounting
        .wrapping_mul(5)
        .wrapping_add(treasure)
        .wrapping_add(0x64)
        .wrapping_mul(drop);
    let base = operation::div_100(scaled as i64) as i32;
    let boosted = operation::div_100(percent.wrapping_add(bonus).wrapping_mul(base) as i64) as i32;
    let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xc, -1)?
        .wrapping_add(0x64)
        .wrapping_mul(boosted);
    let total = operation::div_100(combo as i64) as i32;

    Ok(if total > 0 { total } else { 0 })
}
