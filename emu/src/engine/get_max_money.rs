use crate::{operation, Fault};

use super::{
    get_base_upgrade, get_cat_combo_bonus, get_global_map_id, get_item_count, get_map_rules, get_special_rule_params,
    get_treasure_value, AppContext,
};

const SITE: &str = "get_max_money";

pub fn get_max_money(ctx: &mut AppContext, wallet: usize) -> Result<i32, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0)? {
        let mut cap = *params.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;
        let map_id = get_global_map_id(ctx, 0)?;
        let first_mode = get_map_rules(&ctx.special_rules, map_id)?.ok_or(Fault::NullPointer { site: SITE })?.max_money_item;
        let mut item = 0xcf;
        let mut second_mode = 0i32;

        if first_mode != 1 {
            let map_id = get_global_map_id(ctx, 0)?;

            second_mode = get_map_rules(&ctx.special_rules, map_id)?.ok_or(Fault::NullPointer { site: SITE })?.max_money_item;
            item = 0xf7;
        }

        if first_mode == 1 || second_mode == 2 {
            cap = cap.wrapping_add(get_item_count(ctx, item)?);
        }

        let capped = if cap < 0x98967f { cap } else { 0x98967f };

        return Ok(capped.wrapping_mul(0x64));
    }

    let wallet_upgrade = get_base_upgrade(ctx, 5)?.wrapping_mul(0x2710);
    let cell = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL))?;
    let worker_level = (cell[7] ^ cell[0]) as u32
        | ((cell[6] ^ cell[1]) as u32) << 8
        | ((cell[5] ^ cell[2]) as u32) << 0x10
        | ((cell[4] ^ cell[3]) as u32) << 0x18;
    let scaled = (worker_level.wrapping_mul(5) as i32).wrapping_add(0xa).wrapping_mul(wallet_upgrade.wrapping_add(0x2710));
    let total = get_treasure_value(ctx, &ctx.treasure_store, 4)?.wrapping_add(operation::div_10(scaled as i64) as i32);
    let boosted = get_cat_combo_bonus(ctx, &ctx.combo_store, 9, -1)?.wrapping_add(0x64).wrapping_mul(total);
    let result = operation::div_100(boosted as i64) as i32;

    Ok(if result > 0 { result } else { 0 })
}
