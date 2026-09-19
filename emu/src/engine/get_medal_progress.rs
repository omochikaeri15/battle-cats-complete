use crate::Fault;

use super::{get_user_rank, AppContext};

pub fn get_medal_progress(ctx: &AppContext, kind: i32) -> Result<i32, Fault> {
    match kind as u32 {
        0 => ctx.i32_at(AppContext::MEDAL_MONEY_SPENT),
        1 => ctx.i32_at(AppContext::MEDAL_KIND_1),
        3 => get_user_rank(ctx),
        2 | 4 => Err(Fault::HostMissing { site: "get_medal_progress" }),
        _ => Ok(0),
    }
}
