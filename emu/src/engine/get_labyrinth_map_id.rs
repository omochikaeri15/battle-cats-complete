use crate::Fault;

use super::AppContext;

pub fn get_labyrinth_map_id(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::LABYRINTH_MAP_ID)
}
