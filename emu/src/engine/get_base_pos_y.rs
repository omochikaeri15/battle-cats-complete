use crate::Fault;

use super::{get_pos_y, AppContext};

pub fn get_base_pos_y(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_pos_y(ctx, faction, 0)
}
