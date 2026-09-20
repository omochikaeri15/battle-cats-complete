use crate::Fault;

use super::MapLayout;

pub fn get_stage_set_size(map_layout: &MapLayout, set: i32) -> Result<u64, Fault> {
    if map_layout.points.len() as u64 <= set as i64 as u64 {
        return Err(Fault::index_out_of_range(set as i64, map_layout.points.len() as i64));
    }

    map_layout
        .points
        .get(set as usize)
        .map(|stages| stages.len() as u64)
        .ok_or(Fault::index_out_of_range(set as i64, map_layout.points.len() as i64))
}
