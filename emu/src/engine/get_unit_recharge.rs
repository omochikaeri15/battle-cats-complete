use crate::{Fault, operation};

use super::{
    AppContext, get_base_upgrade, get_button_unit_form, get_button_unit_id, get_cat_combo_bonus,
    get_global_map_id, get_special_rule_params, get_treasure_value, get_unit_rarity, stat_cooldown,
};

pub fn get_unit_recharge(ctx: &mut AppContext, faction: i32, button: i32) -> Result<i32, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 1)? {
        return params.first().copied().ok_or(Fault::index_out_of_range(0, 0));
    }

    let unit_id = get_button_unit_id(ctx, faction, button)?;
    let form = get_button_unit_form(ctx, faction, button)?;
    let cooldown = stat_cooldown(ctx, 0, unit_id, form)?;
    let upgrade = get_base_upgrade(ctx, 7)?.wrapping_mul(3);
    let treasure = get_treasure_value(ctx, &ctx.treasure_store, 0xb)?;
    let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xb, unit_id)?;
    let reduction = treasure.wrapping_add(upgrade.wrapping_mul(2));
    let reduced = operation::div_neg_100(combo.wrapping_mul(reduction))
        .wrapping_add(cooldown.wrapping_sub(reduction));
    let mut recharge = if reduced >= 0x3d { reduced } else { 0x3c };
    let map_id = get_global_map_id(ctx, 0)?;

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 6)? {
        let rarity = get_unit_rarity(ctx, unit_id)?;
        let percent = *params
            .get(rarity as i64 as usize)
            .ok_or(Fault::index_out_of_range(rarity as i64, params.len() as i64))?;

        recharge = operation::div_100(recharge.wrapping_mul(percent));
    }

    Ok(recharge)
}
