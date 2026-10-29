use crate::Fault;

use super::{AppContext, set_soulstrike};

pub fn set_base_soulstrike(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_soulstrike(ctx, faction, 0, value)
}
