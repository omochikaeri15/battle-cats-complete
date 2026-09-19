use std::collections::BTreeMap;

use super::RewardDef;

pub fn get_point_rewards(defs: &BTreeMap<i32, Vec<RewardDef>>, point_id: i32) -> Option<&Vec<RewardDef>> {
    defs.get(&point_id)
}
