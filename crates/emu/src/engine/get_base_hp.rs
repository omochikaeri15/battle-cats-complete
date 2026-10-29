use crate::Fault;

use super::{AppContext, get_hp};

pub fn get_base_hp(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_hp(ctx, faction, 0)
}
