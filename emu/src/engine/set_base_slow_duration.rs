use crate::Fault;

use super::{set_slow_duration, AppContext};

pub fn set_base_slow_duration(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_slow_duration(ctx, faction, 0, value)
}
