use crate::Fault;

use super::{get_scene_id, get_special_rule, special_rules_at, AppContext, SpecialRuleStore};

pub fn get_special_rule_params<'a>(
    ctx: &AppContext,
    store: &'a SpecialRuleStore,
    map_id: i32,
    rule: i32,
) -> Result<Option<&'a Vec<i32>>, Fault> {
    if !get_special_rule(ctx, store, map_id, rule)? {
        return Ok(None);
    }

    if get_scene_id(ctx)? == 0x12c
        && store.fever_count > 0
        && special_rules_at(store, &map_id)?.fever.contains_key(&rule)
    {
        return special_rules_at(store, &map_id)?
            .fever
            .get(&rule)
            .map(Some)
            .ok_or(Fault::KeyNotFound { site: "get_special_rule_params", key: rule as i64 });
    }

    special_rules_at(store, &map_id)?
        .normal
        .get(&rule)
        .map(Some)
        .ok_or(Fault::KeyNotFound { site: "get_special_rule_params", key: rule as i64 })
}
