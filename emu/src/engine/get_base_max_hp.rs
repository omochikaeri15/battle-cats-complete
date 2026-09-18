use crate::Fault;

use super::{get_max_hp, AppContext};

pub fn get_base_max_hp(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    get_max_hp(ctx, faction, 0)
}
