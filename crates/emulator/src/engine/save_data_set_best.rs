use super::PointRecord;

pub fn save_data_set_best(record: &mut PointRecord, key: i32, stage: i32, value: i32) {
    let existing = record
        .best
        .get(&key)
        .and_then(|stages| stages.get(&stage))
        .copied();
    let value = existing.map_or(
        value,
        |existing| if existing > value { existing } else { value },
    );

    *record
        .best
        .entry(key)
        .or_default()
        .entry(stage)
        .or_insert(0) = value;
}
