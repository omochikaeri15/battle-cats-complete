use crate::{Fault, ops};

use super::{
    AppContext, CatStats, get_cat_combo_values, get_global_map_id, get_map_cost_multiplier,
    get_orb_value_max, get_special_rule_params, get_talent_value, get_unit_rarity, max_i32,
    orb_deploy_condition, read_flag,
};

pub fn get_deploy_cost(
    ctx: &mut AppContext,
    unit_id: i32,
    form: i32,
    apply_rules: i32,
    slot: i32,
) -> Result<i32, Fault> {
    if apply_rules != 0 {
        let map_id = get_global_map_id(ctx, 0)?;

        if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 4)? {
            let flat = *params.first().ok_or(Fault::index_out_of_range(0, 0))?;

            return Ok(flat.wrapping_mul(0x64));
        }
    }

    let mut base = 0i32;

    if read_flag(ctx, AppContext::faction_flags(0))? & 1 != 0 {
        let column = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::EOC1_COST))?;
        let discount = get_talent_value(ctx, 0, unit_id, form, 0x19, 0)?;

        base =
            ops::div_100(max_i32(discount.wrapping_mul(-100).wrapping_add(column), 0) as i64)
                as i32;
    }

    let tiered = ctx
        .i32_at(AppContext::CHAPTER_COST_TIER)?
        .wrapping_add(2)
        .wrapping_mul(base);
    let mut cost = (((tiered as u32 >> 0x1f) as i32).wrapping_add(tiered) >> 1).wrapping_mul(0x64);

    if apply_rules as u8 != 0 {
        let map_id = get_global_map_id(ctx, 0)?;

        if get_map_cost_multiplier(&ctx.map_options.cost_multipliers, map_id) != 0 {
            let mut base = 0i32;

            if read_flag(ctx, AppContext::faction_flags(0))? & 1 != 0 {
                let column =
                    ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::EOC1_COST))?;
                let discount = get_talent_value(ctx, 0, unit_id, form, 0x19, 0)?;

                base = ops::div_100(max_i32(
                    discount.wrapping_mul(-100).wrapping_add(column),
                    0,
                ) as i64) as i32;
            }

            let map_id = get_global_map_id(ctx, 0)?;

            cost = get_map_cost_multiplier(&ctx.map_options.cost_multipliers, map_id).wrapping_mul(base);
        }
    }

    if slot != -1 {
        if orb_deploy_condition(ctx, AppContext::faction_flags(0), 0, slot)? {
            let orb = get_orb_value_max(ctx, unit_id, 0x16, 0, 0)?;

            cost = (ops::div_10000(0x64i32.wrapping_sub(orb).wrapping_mul(cost) as i64)
                as i32)
                .wrapping_mul(0x64);
        }

        if apply_rules as u8 == 0 {
            return Ok(if cost > 0 { cost } else { 0 });
        }

        for value in get_cat_combo_values(ctx, &ctx.combo_store, 0x1b, unit_id)? {
            cost = (ops::div_10000(0x64i32.wrapping_sub(value).wrapping_mul(cost) as i64)
                as i32)
                .wrapping_mul(0x64);
        }
    }

    if apply_rules as u8 != 0 {
        let map_id = get_global_map_id(ctx, 0)?;

        if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 5)? {
            let rarity = get_unit_rarity(ctx, unit_id)? as i64;
            let percent = *params.get(rarity as usize).ok_or(Fault::index_out_of_range(rarity, params.len() as i64))?;

            cost = ops::div_100(cost.wrapping_mul(percent) as i64) as i32;
        }
    }

    Ok(if cost > 0 { cost } else { 0 })
}
