use super::MapLayout;

pub fn map_layout_set_points(record: &mut MapLayout, index: i32, points: &[u64]) {
    if let Some(slot) = record.points.get_mut(index as i64 as usize) {
        *slot = points.to_vec();
    }
}
