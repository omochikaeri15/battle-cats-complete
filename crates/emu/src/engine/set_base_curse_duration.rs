use crate::Fault;

use super::{AppContext, set_curse_duration};

pub fn set_base_curse_duration(
    ctx: &mut AppContext,
    faction: i32,
    value: i32,
) -> Result<(), Fault> {
    set_curse_duration(ctx, faction, 0, value)
}
