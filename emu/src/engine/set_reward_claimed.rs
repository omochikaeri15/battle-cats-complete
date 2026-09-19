use std::collections::BTreeMap;

pub fn set_reward_claimed(
    claimed: &mut BTreeMap<i32, Vec<i32>>,
    point_id: i32,
    reward_id: i32,
    value: i32,
) {
    if value != 0 {
        let ids = claimed.entry(point_id).or_default();

        if !ids.contains(&reward_id) {
            ids.push(reward_id);
        }

        return;
    }

    if let Some(ids) = claimed.get_mut(&point_id) {
        ids.retain(|id| *id != reward_id);
    }
}
