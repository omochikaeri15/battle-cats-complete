use crate::{Fault, engine::AppContext};

const MEDAL_PROGRESS_CAP: i32 = 2_000_000_000;

pub fn fill_dummy_save(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::MEDAL_MONEY_0, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_1, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_4, MEDAL_PROGRESS_CAP)
}
