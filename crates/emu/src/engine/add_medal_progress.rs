use crate::Fault;

use super::{AppContext, get_medal_progress};

pub fn add_medal_progress(ctx: &mut AppContext, kind: i32, amount: i32) -> Result<(), Fault> {
    let total = get_medal_progress(ctx, kind)?.wrapping_add(amount);
    let capped = if total < 0x77359400 {
        total
    } else {
        0x77359400
    };

    match kind {
        4 => ctx.set_i32_at(AppContext::MEDAL_MONEY_4, capped),
        1 => ctx.set_i32_at(AppContext::MEDAL_MONEY_1, capped),
        0 => ctx.set_i32_at(AppContext::MEDAL_MONEY_0, capped),
        _ => Ok(()),
    }
}
