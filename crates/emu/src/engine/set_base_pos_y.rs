use crate::Fault;

use super::{AppContext, set_pos_y};

pub fn set_base_pos_y(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_pos_y(ctx, faction, 0, value)
}
