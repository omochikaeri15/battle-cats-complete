use crate::Fault;

use super::{AppContext, Labyrinth};

pub fn get_cleared_count(ctx: &AppContext, obj: usize) -> Result<i32, Fault> {
    ctx.i32_at(obj.wrapping_add(Labyrinth::CLEARED_COUNT))
}
