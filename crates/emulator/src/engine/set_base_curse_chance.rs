use crate::Fault;

use super::{AppContext, set_curse_chance};

pub fn set_base_curse_chance(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_curse_chance(ctx, faction, 0, value)
}
