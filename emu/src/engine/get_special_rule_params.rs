use crate::Fault;

use super::{AppContext, SpecialRuleStore, get_scene_id, get_special_rule, special_rules_at};

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
            .ok_or(Fault::key_not_found(rule as i64));
    }

    special_rules_at(store, &map_id)?
        .normal
        .get(&rule)
        .map(Some)
        .ok_or(Fault::key_not_found(rule as i64))
}
