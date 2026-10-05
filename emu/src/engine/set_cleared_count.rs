use crate::Fault;

use super::{AppContext, Labyrinth};

pub fn set_cleared_count(ctx: &mut AppContext, obj: usize, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(obj.wrapping_add(Labyrinth::CLEARED_COUNT), value)
}
