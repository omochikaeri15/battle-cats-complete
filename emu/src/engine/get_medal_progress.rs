use crate::Fault;

use super::{AppContext, get_user_rank};

pub fn get_medal_progress(ctx: &AppContext, kind: i32) -> Result<i32, Fault> {
    match kind as u32 {
        0 => ctx.i32_at(AppContext::MEDAL_MONEY_0),
        1 => ctx.i32_at(AppContext::MEDAL_MONEY_1),
        3 => get_user_rank(ctx),
        2 | 4 => Err(Fault::host_missing()),
        _ => Ok(0),
    }
}
