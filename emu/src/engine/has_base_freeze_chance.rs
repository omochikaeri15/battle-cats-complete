use crate::Fault;

use super::{has_freeze_chance, AppContext};

pub fn has_base_freeze_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_freeze_chance(ctx, faction, 0)
}
