use crate::Fault;

use super::{get_cannon_save_level, AppContext};

pub fn get_cannon_base_level(ctx: &AppContext) -> Result<i32, Fault> {
    get_cannon_save_level(ctx, 0)
}
