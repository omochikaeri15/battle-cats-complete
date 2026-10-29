use crate::Fault;

use super::AppContext;

pub fn map_item_cost_flag(
    ctx: &mut AppContext,
    map_type: i32,
    map_index: i32,
) -> Result<bool, Fault> {
    if map_type == -10 {
        return Ok(true);
    }

    let options = ctx
        .map_stage_options
        .entry(map_type)
        .or_insert_with(|| vec![[0; 4]; 0x1f4]);
    let row = options
        .get(map_index as i64 as usize)
        .ok_or(Fault::index_out_of_range(map_index as i64, 0x1f4))?;

    Ok(row[3] != 0)
}
