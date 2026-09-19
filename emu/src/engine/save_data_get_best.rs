use super::PointRecord;

pub fn save_data_get_best(record: &PointRecord, key: i32, stage: i32) -> i32 {
    record.best.get(&key).and_then(|stages| stages.get(&stage)).copied().unwrap_or(0)
}
