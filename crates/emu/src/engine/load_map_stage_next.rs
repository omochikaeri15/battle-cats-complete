use crate::Fault;

use super::{AppContext, load_map_stage_csv};

pub fn load_map_stage_next(
    ctx: &mut AppContext,
    map: i32,
    stage: i32,
    check_pack: u8,
    saga: u8,
) -> Result<bool, Fault> {
    load_map_stage_csv(ctx, map, stage, check_pack, saga, 0, 1)
}
