use crate::Fault;

use super::{AppContext, set_max_hp};

pub fn set_base_max_hp(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_max_hp(ctx, faction, 0, value)
}
