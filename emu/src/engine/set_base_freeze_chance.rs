use crate::Fault;

use super::{set_freeze_chance, AppContext};

pub fn set_base_freeze_chance(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_freeze_chance(ctx, faction, 0, value)
}
