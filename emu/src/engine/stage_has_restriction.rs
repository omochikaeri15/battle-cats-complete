use std::collections::BTreeMap;

use crate::{operation, Fault};

use super::{get_special_rule, AppContext};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StageRestriction {
    pub unknown_0: i32,
    pub rarity_mask: i32,
    pub deploy_limit: i32,
    pub rows: i32,
    pub min_cost: i32,
    pub max_cost: i32,
    pub group_id: i32,
}

pub fn stage_has_restriction(ctx: &AppContext, store: &BTreeMap<i32, StageRestriction>, stage_id: i32) -> Result<bool, Fault> {
    let map_id = operation::div_1000(stage_id);

    if get_special_rule(ctx, &ctx.special_rules, map_id, 2)? {
        return Ok(true);
    }

    if get_special_rule(ctx, &ctx.special_rules, map_id, 3)? {
        return Ok(true);
    }

    if get_special_rule(ctx, &ctx.special_rules, map_id, 7)? {
        return Ok(true);
    }

    Ok(store.contains_key(&stage_id))
}
