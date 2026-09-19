use crate::Fault;

use super::{get_cannon_save_value, AppContext};

pub fn get_style_part_level(ctx: &AppContext, part_id: i32) -> Result<i32, Fault> {
    Ok(get_cannon_save_value(ctx, part_id, 2)?.wrapping_sub(1))
}
