use crate::Fault;

use super::{AppContext, has_slow_chance};

pub fn has_base_slow_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_slow_chance(ctx, faction, 0)
}
