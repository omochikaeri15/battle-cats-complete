use crate::Fault;

use super::{AppContext, Debris};

pub fn debris_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x38usize {
        let timer = AppContext::CAT_DEBRIS.wrapping_add(record.wrapping_mul(AppContext::DEBRIS_STRIDE)).wrapping_add(Debris::TIMER);
        let value = ctx.i32_at(timer)?;

        ctx.set_i32_at(timer, if value <= 0 { 0 } else { value.wrapping_sub(1) })?;
    }

    for record in 0..0x38usize {
        let timer = AppContext::ENEMY_DEBRIS.wrapping_add(record.wrapping_mul(AppContext::DEBRIS_STRIDE)).wrapping_add(Debris::TIMER);
        let value = ctx.i32_at(timer)?;

        ctx.set_i32_at(timer, if value <= 0 { 0 } else { value.wrapping_sub(1) })?;
    }

    Ok(())
}
