use crate::Fault;

use super::{get_cannon_save_level, AppContext};

pub fn get_effect_part_level(ctx: &AppContext, part_id: i32) -> Result<i32, Fault> {
    get_cannon_save_level(ctx, part_id)
}
