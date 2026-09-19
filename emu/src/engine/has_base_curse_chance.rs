use crate::Fault;

use super::{AppContext, has_curse_chance};

pub fn has_base_curse_chance(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    has_curse_chance(ctx, faction, 0)
}
