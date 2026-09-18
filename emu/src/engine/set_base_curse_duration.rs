use crate::Fault;

use super::{set_curse_duration, AppContext};

pub fn set_base_curse_duration(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_curse_duration(ctx, faction, 0, value)
}
