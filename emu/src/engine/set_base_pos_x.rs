use crate::Fault;

use super::{AppContext, set_pos_x};

pub fn set_base_pos_x(ctx: &mut AppContext, faction: i32, x: i32) -> Result<(), Fault> {
    set_pos_x(ctx, faction, 0, x)
}
