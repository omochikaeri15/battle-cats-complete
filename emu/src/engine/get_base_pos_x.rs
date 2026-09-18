use crate::Fault;

use super::{get_pos_x, AppContext};

pub fn get_base_pos_x(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_pos_x(ctx, faction, 0)
}
