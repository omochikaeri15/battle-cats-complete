use crate::Fault;

use super::{AppContext, is_touchable};

pub fn is_touchable_thunk(
    ctx: &AppContext,
    faction: i32,
    slot: i32,
    attacker: i32,
) -> Result<bool, Fault> {
    is_touchable(ctx, faction, slot, attacker)
}
