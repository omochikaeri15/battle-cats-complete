use crate::Fault;

use super::{get_base_upgrade, AppContext};

pub fn get_worker_upgrade_level(ctx: &mut AppContext) -> Result<i32, Fault> {
    get_base_upgrade(ctx, 4)
}
