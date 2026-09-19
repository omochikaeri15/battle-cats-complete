use crate::Fault;

use super::{AppContext, set_slow_duration};

pub fn set_base_slow_duration(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_slow_duration(ctx, faction, 0, value)
}
