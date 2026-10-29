use crate::Fault;

use super::AppContext;

pub fn get_stage_variant(ctx: &AppContext) -> Result<i32, Fault> {
    if ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0 {
        return Ok(1);
    }

    if ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0 {
        return Ok(2);
    }

    Ok((ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? as i32).wrapping_mul(3))
}
