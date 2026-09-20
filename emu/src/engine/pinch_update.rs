use crate::{Fault, ops};

use super::{AppContext, Pinch};

pub fn pinch_update(ctx: &mut AppContext, pinch: usize) -> Result<(), Fault> {
    let released = ctx.u8_at(pinch.wrapping_add(Pinch::FIRST_DOWN))? == 0
        || ctx.u8_at(pinch.wrapping_add(Pinch::SECOND_DOWN))? == 0;

    if ctx.u8_at(pinch.wrapping_add(Pinch::ACTIVE))? != 0 {
        if released {
            ctx.set_block_at::<2>(pinch.wrapping_add(Pinch::ACTIVE), [0, 0])?;
            ctx.set_block_at::<1>(pinch.wrapping_add(Pinch::SECOND_DOWN), [0])?;

            return Ok(());
        }

        ctx.set_i32_at(
            pinch.wrapping_add(Pinch::PREV_DISTANCE),
            ctx.i32_at(pinch.wrapping_add(Pinch::DISTANCE))?,
        )?;

        let across =
            ctx.i32_at(pinch.wrapping_add(Pinch::FIRST_X))?
                .wrapping_sub(ctx.i32_at(pinch.wrapping_add(Pinch::SECOND_X))?) as f64;
        let down =
            ctx.i32_at(pinch.wrapping_add(Pinch::FIRST_Y))?
                .wrapping_sub(ctx.i32_at(pinch.wrapping_add(Pinch::SECOND_Y))?) as f64;
        let distance = (down * down + across * across).sqrt();

        ctx.set_i32_at(
            pinch.wrapping_add(Pinch::DISTANCE),
            ops::cvttsd2si(distance),
        )?;

        return Ok(());
    }

    if released {
        return Ok(());
    }

    ctx.set_block_at::<1>(pinch.wrapping_add(Pinch::ACTIVE), [1])?;

    let fingers = ctx.block_at::<16>(pinch.wrapping_add(Pinch::FIRST_X))?;

    ctx.set_block_at::<16>(pinch.wrapping_add(Pinch::START), fingers)?;

    let across = ctx
        .i32_at(pinch.wrapping_add(Pinch::FIRST_X))?
        .wrapping_sub(ctx.i32_at(pinch.wrapping_add(Pinch::SECOND_X))?) as f64;
    let down = ctx
        .i32_at(pinch.wrapping_add(Pinch::FIRST_Y))?
        .wrapping_sub(ctx.i32_at(pinch.wrapping_add(Pinch::SECOND_Y))?) as f64;
    let distance = ops::cvttsd2si((down * down + across * across).sqrt());

    ctx.set_i32_at(pinch.wrapping_add(Pinch::PREV_DISTANCE), distance)?;
    ctx.set_i32_at(pinch.wrapping_add(Pinch::DISTANCE), distance)
}
