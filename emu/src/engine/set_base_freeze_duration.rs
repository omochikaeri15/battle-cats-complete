use crate::Fault;

use super::{set_freeze_duration, AppContext};

pub fn set_base_freeze_duration(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_freeze_duration(ctx, faction, 0, value)
}
