use crate::Fault;

use super::{AppContext, get_global_map_id, get_special_rule_params};

pub fn fever_time_fraction(ctx: &mut AppContext) -> Result<f64, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;
    let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xc)? else {
        return Ok(0.0);
    };
    let limit = *params.get(2).ok_or(Fault::out_of_range())?;

    Ok(ctx.special_rules.fever_count as f64 / limit as f64)
}
