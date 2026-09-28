use crate::Fault;

use super::MapLayout;

pub fn map_layout_set_points(record: &mut MapLayout, index: i32, points: &[u64]) -> Result<(), Fault> {
    let limit = record.points.len() as i64;
    let slot = record.points.get_mut(index as i64 as usize).ok_or(Fault::index_out_of_range(index as i64, limit))?;

    *slot = points.to_vec();

    Ok(())
}
