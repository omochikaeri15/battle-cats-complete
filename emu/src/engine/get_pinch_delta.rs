use crate::Fault;

use super::{AppContext, Pinch};

pub fn get_pinch_delta(ctx: &AppContext, pinch: usize) -> Result<i32, Fault> {
    let delta = ctx.i32_at(pinch.wrapping_add(Pinch::DISTANCE))?.wrapping_sub(ctx.i32_at(pinch.wrapping_add(Pinch::PREV_DISTANCE))?);

    if ctx.u8_at(pinch.wrapping_add(Pinch::ACTIVE))? != 0 {
        return Ok(delta);
    }

    Ok(0)
}
