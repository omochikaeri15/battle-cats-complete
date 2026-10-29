use std::collections::BTreeMap;

pub fn is_reward_claimed(claimed: &BTreeMap<i32, Vec<i32>>, point_id: i32, reward_id: i32) -> bool {
    claimed
        .get(&point_id)
        .is_some_and(|ids| ids.contains(&reward_id))
}
