use crate::{operation, Fault};

use super::{get_base_upgrade, get_cat_combo_bonus, get_treasure_value, AppContext};

pub fn get_money_increment(ctx: &mut AppContext, wallet: usize) -> Result<i32, Fault> {
    let rate = get_base_upgrade(ctx, 4)?.wrapping_mul(5).wrapping_mul(2).wrapping_add(0x19);
    let cell = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL))?;
    let worker_level = (((cell[7] ^ cell[0]) as u32 | ((cell[6] ^ cell[1]) as u32) << 8 | ((cell[5] ^ cell[2]) as u32) << 0x10) as i32)
        .wrapping_add((((cell[4] ^ cell[3]) as u32) << 0x18) as i32);
    let earned = operation::div_10(worker_level.wrapping_add(0xa).wrapping_mul(rate) as i64) as i32;
    let total = get_treasure_value(ctx, &ctx.treasure_store, 3)?.wrapping_add(earned);
    let boosted = get_cat_combo_bonus(ctx, &ctx.combo_store, 8, -1)?.wrapping_add(0x64).wrapping_mul(total);

    Ok(operation::div_100(boosted as i64) as i32)
}
