use crate::{Fault, operation};

use super::{AppContext, get_max_hp};

pub fn get_base_max_hp_div_20(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    Ok(operation::div_20(get_max_hp(ctx, faction, 0)? as i64) as i32)
}
