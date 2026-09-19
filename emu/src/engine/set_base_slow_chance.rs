use crate::Fault;

use super::{AppContext, set_slow_chance};

pub fn set_base_slow_chance(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_slow_chance(ctx, faction, 0, value)
}
