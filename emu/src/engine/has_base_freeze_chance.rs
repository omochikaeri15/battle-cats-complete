use crate::Fault;

use super::{AppContext, has_freeze_chance};

pub fn has_base_freeze_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_freeze_chance(ctx, faction, 0)
}
