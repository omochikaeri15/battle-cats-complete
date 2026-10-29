use crate::Fault;

use super::{AppContext, vibrate};

pub fn vibration_sample(ctx: &mut AppContext) -> Result<(), Fault> {
    let power = ctx.vibration.power;
    let time = ctx.vibration.time;
    let tail = ctx.vibration.tail;

    vibrate(ctx, power, time, tail)
}
