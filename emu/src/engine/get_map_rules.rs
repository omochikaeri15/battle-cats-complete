use crate::Fault;

use super::{special_rules_at, MapRules, SpecialRuleStore};

pub fn get_map_rules(store: &SpecialRuleStore, map_id: i32) -> Result<Option<&MapRules>, Fault> {
    if !store.maps.contains_key(&map_id) {
        return Ok(None);
    }

    special_rules_at(store, &map_id).map(Some)
}
