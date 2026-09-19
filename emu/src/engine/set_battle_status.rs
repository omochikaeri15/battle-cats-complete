use crate::Fault;

use super::{update_keep_awake, AppContext};

pub fn set_battle_status(ctx: &mut AppContext, status: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::BATTLE_STATUS, status)?;

    update_keep_awake(ctx, 1)
}
