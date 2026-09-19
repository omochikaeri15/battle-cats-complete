use crate::Fault;

use super::AppContext;

pub fn get_labyrinth_floor_reached(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::LABYRINTH_FLOOR_REACHED)
}
