use crate::Fault;

use super::AppContext;

pub fn get_battle_status(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::BATTLE_STATUS)
}
