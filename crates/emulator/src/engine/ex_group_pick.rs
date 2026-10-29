use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, call_rng};

#[derive(Clone, Default)]
pub struct ExGroup {
    pub group: i32,
    pub indices: Vec<i32>,
    pub thresholds: Vec<i32>,
}

pub fn ex_group_pick(ctx: &mut AppContext, map: i32, stage: i32) -> Result<i32, Fault> {
    let key = map.wrapping_mul(100).wrapping_add(stage);

    if !ctx.ex_groups.contains_key(&key) {
        return Ok(-1);
    }

    let roll = call_rng(ctx, 100);
    let groups: &BTreeMap<i32, ExGroup> = &ctx.ex_groups;
    let mut index = 0usize;

    loop {
        let group = groups.get(&key).ok_or(Fault::key_not_found(key as i64))?;

        if group.thresholds.len() <= index {
            return Ok(-1);
        }

        let group = groups.get(&key).ok_or(Fault::key_not_found(key as i64))?;
        let threshold = *group
            .thresholds
            .get(index)
            .ok_or(Fault::out_of_range())?;

        if roll < threshold {
            let group = groups.get(&key).ok_or(Fault::key_not_found(key as i64))?;

            return group
                .indices
                .get(index)
                .copied()
                .ok_or(Fault::out_of_range());
        }

        index += 1;
    }
}
