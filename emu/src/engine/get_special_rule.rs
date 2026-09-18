use crate::Fault;

use super::{get_scene_id, special_rules_at, AppContext, SpecialRuleStore};

pub fn get_special_rule(ctx: &AppContext, store: &SpecialRuleStore, map_id: i32, rule: i32) -> Result<bool, Fault> {
    if !store.maps.contains_key(&map_id) {
        return Ok(false);
    }

    if special_rules_at(store, &map_id)?.normal.contains_key(&rule) {
        return Ok(true);
    }

    if get_scene_id(ctx)? == 0x12c && store.fever_count > 0 {
        return Ok(special_rules_at(store, &map_id)?.fever.contains_key(&rule));
    }

    Ok(false)
}
