use crate::{engine::AppContext, Fault};

const MEDAL_PROGRESS_CAP: i32 = 2_000_000_000;

pub fn fill_dummy_save(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::MEDAL_MONEY_SPENT, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_KIND_1, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_KIND_4, MEDAL_PROGRESS_CAP)
}
