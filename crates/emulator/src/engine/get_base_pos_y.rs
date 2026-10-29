use crate::Fault;

use super::{AppContext, get_pos_y};

pub fn get_base_pos_y(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_pos_y(ctx, faction, 0)
}
