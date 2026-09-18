use crate::Fault;

use super::{set_max_hp, AppContext};

pub fn set_base_max_hp(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_max_hp(ctx, faction, 0, value)
}
