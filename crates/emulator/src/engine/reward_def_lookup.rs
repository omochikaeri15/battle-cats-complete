use std::collections::BTreeMap;

use super::{PointEventReward, PointReward};

pub fn reward_def_lookup(
    defs: &BTreeMap<i32, PointEventReward>,
    group: i32,
    id: i32,
) -> Option<&PointReward> {
    defs.get(&group)?;

    let mut index = 0usize;

    loop {
        let list = &defs.get(&group)?.rewards;

        if list.len() <= index {
            return None;
        }

        if list.get(index)?.reward_id == id {
            return list.get(index);
        }

        index += 1;
    }
}
