use crate::Fault;

use super::{has_curse_chance, AppContext};

pub fn has_base_curse_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_curse_chance(ctx, faction, 0)
}
