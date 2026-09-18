use crate::Fault;

use super::{has_slow_chance, AppContext};

pub fn has_base_slow_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_slow_chance(ctx, faction, 0)
}
