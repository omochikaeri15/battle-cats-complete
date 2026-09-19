use crate::Fault;

use super::{AppContext, get_pos_x};

pub fn get_base_pos_x(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_pos_x(ctx, faction, 0)
}
