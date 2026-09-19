use crate::Fault;

use super::{get_base_upgrade, AppContext};

pub fn compute_base_level(ctx: &mut AppContext) -> Result<i32, Fault> {
    Ok(get_base_upgrade(ctx, 2)?.wrapping_add(3))
}
