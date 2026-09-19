use std::collections::BTreeMap;

use super::AppContext;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ShakeRecord {
    pub amplitude_from: i32,
    pub amplitude_to: i32,
    pub duration: i32,
    pub priority: i32,
    pub count: i32,
    pub until: i32,
    pub since: i32,
    pub reset_frame: i32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BaseShake {
    pub records: BTreeMap<i32, ShakeRecord>,
    pub id: i32,
    pub frame: i32,
    pub unknown_2: i32,
    pub offset: i32,
}

impl Default for BaseShake {
    fn default() -> Self {
        Self { records: BTreeMap::new(), id: -1, frame: 0, unknown_2: 0, offset: 0 }
    }
}

pub fn base_shake_start(ctx: &mut AppContext, id: i32) {
    let shake = &mut ctx.base_shake;

    if shake.records.is_empty() {
        return;
    }

    if !shake.records.contains_key(&id) {
        return;
    }

    let current = shake.id;

    if current != -1 {
        let current_priority = shake.records.entry(current).or_default().priority;

        if current_priority < shake.records.entry(id).or_default().priority && shake.frame == 0 {
            return;
        }

        if shake.frame > 0 {
            return;
        }
    }

    if shake.records.entry(id).or_default().reset_frame > 0 {
        let since = shake.records.entry(id).or_default().since;

        if since >= shake.records.entry(id).or_default().reset_frame {
            let record = shake.records.entry(id).or_default();

            record.since = 0;
            record.count = 0;
        }
    }

    if shake.records.entry(id).or_default().count == 0 {
        shake.offset = 0;
        shake.frame = 0;
        shake.unknown_2 = 0;
        shake.id = id;
    }

    let record = shake.records.entry(id).or_default();
    let count = record.count;

    record.count = count.wrapping_add(1);

    if count < shake.records.entry(id).or_default().until {
        return;
    }

    let record = shake.records.entry(id).or_default();

    record.since = 0;
    record.count = 0;
}
