use crate::Fault;

use super::{get_scene_id, AppContext};

pub fn scene_transition_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_scene_id(ctx)? == 0x12c {
        return Ok(());
    }

    for slot in 0..10usize {
        let cell = AppContext::SCENE_LATCHES + slot * 4;

        if ctx.i32_at(cell)? == 1 {
            ctx.set_i32_at(cell, 2)?;
        }
    }

    Ok(())
}
