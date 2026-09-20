use crate::Fault;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapData {
    pub stage_sets: Vec<Vec<u64>>,
}

pub fn get_stage_set_size(map_data: &MapData, set: i32) -> Result<u64, Fault> {
    if map_data.stage_sets.len() as u64 <= set as i64 as u64 {
        return Err(Fault::index_out_of_range(set as i64, map_data.stage_sets.len() as i64));
    }

    map_data
        .stage_sets
        .get(set as usize)
        .map(|stages| stages.len() as u64)
        .ok_or(Fault::index_out_of_range(set as i64, map_data.stage_sets.len() as i64))
}
