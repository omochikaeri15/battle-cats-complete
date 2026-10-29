use crate::Fault;

use super::{AppContext, get_soulstrike};

pub fn get_base_soulstrike(ctx: &AppContext, faction: i32) -> Result<bool, Fault> {
    get_soulstrike(ctx, faction, 0)
}
