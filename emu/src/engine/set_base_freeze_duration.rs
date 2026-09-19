use crate::Fault;

use super::{AppContext, set_freeze_duration};

pub fn set_base_freeze_duration(
    ctx: &mut AppContext,
    faction: i32,
    value: i32,
) -> Result<(), Fault> {
    set_freeze_duration(ctx, faction, 0, value)
}
