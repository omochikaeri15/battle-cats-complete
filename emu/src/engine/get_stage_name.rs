use crate::Fault;

use super::AppContext;

pub fn get_stage_name(ctx: &mut AppContext, map_type: i32, map_index: i32, stage: i32) -> Result<Vec<u8>, Fault> {
    Ok(ctx.text_renderer().ok_or(Fault::HostMissing { site: "get_stage_name" })?.stage_name(map_type, map_index, stage))
}
