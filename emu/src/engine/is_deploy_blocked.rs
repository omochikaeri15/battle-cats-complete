use crate::Fault;

use super::{get_button_unit_id, get_effective_deploy_cost, get_global_map_id, get_money, get_special_rule_params, get_unit_rarity, AppContext};

pub fn is_deploy_blocked(ctx: &mut AppContext, slot: i32) -> Result<bool, Fault> {
    const SITE: &str = "is_deploy_blocked";

    let map_id = get_global_map_id(ctx, 0)?;

    if get_special_rule_params(ctx, &ctx.special_rules, map_id, 0)?.is_some() {
        let money = get_money(ctx, AppContext::faction_flags(0))?;

        return Ok(money < get_effective_deploy_cost(ctx, 0, slot)?);
    }

    let map_id = get_global_map_id(ctx, 0)?;

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 3)? {
        let rarity = get_unit_rarity(ctx, get_button_unit_id(ctx, 0, slot)?)? as i64;
        let cap = *params.get(rarity as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rarity, limit: params.len() as i64 })?;

        if cap > 0 && ctx.i32_at((rarity * 4 + AppContext::DEPLOY_LIMIT_RARITY_COUNTS as i64) as usize)? >= cap {
            return Ok(true);
        }
    }

    let map_id = get_global_map_id(ctx, 0)?;

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 7)? {
        let cap = *params.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

        return Ok(ctx.i32_at(AppContext::DEPLOY_LIMIT_TOTAL)? >= cap);
    }

    Ok(false)
}
