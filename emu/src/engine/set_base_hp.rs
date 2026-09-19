use crate::Fault;

use super::{AppContext, set_hp};

pub fn set_base_hp(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_hp(ctx, faction, 0, value)
}
