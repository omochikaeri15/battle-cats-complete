use crate::Fault;

use super::{AppContext, level_cell_base, level_cell_plus};

pub fn get_unit_level(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    let cell = AppContext::UNIT_LEVELS.wrapping_add((unit_id as i64 as usize).wrapping_mul(8));
    let plus = level_cell_plus(ctx, cell)?;

    Ok(level_cell_base(ctx, cell)?
        .wrapping_add(plus)
        .wrapping_add(1))
}
