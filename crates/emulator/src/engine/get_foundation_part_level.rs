use crate::Fault;

use super::{AppContext, get_cannon_save_value};

pub fn get_foundation_part_level(ctx: &AppContext, part_id: i32) -> Result<i32, Fault> {
    Ok(get_cannon_save_value(ctx, part_id, 1)?.wrapping_sub(1))
}
