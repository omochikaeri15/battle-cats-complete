use crate::Fault;

use super::{AppContext, draw_context, draw_model, maanim_execute};

pub fn skill_up_draw(
    ctx: &mut AppContext,
    x: i32,
    y: i32,
    frame: i32,
    faction: i32,
) -> Result<(), Fault> {
    let (model, anim) = if faction != 0 {
        (&mut ctx.skill_up_e_model, &ctx.skill_up_e_anim)
    } else {
        (&mut ctx.skill_up_model, &ctx.skill_up_anim)
    };

    maanim_execute(model, Some(anim), frame, 0)?;
    draw_model(draw_context(&mut ctx.draw)?, model, x, y);

    Ok(())
}
