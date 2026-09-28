use std::collections::BTreeMap;

use super::PointEventReward;

pub fn get_point_rewards(
    defs: &BTreeMap<i32, PointEventReward>,
    point_id: i32,
) -> Option<&PointEventReward> {
    defs.get(&point_id)
}
