use crate::Fault;

use super::{draw_context, draw_model, maanim_execute, AppContext};

const SITE: &str = "skill_curse_draw";

pub fn skill_curse_draw(ctx: &mut AppContext, x: i32, y: i32, frame: i32, faction: i32) -> Result<(), Fault> {
    if faction as u32 >= 2 {
        return Err(Fault::OutOfRange { site: SITE });
    }

    let (model, anim) = if faction == 0 { (&mut ctx.skill_curse_model, &ctx.skill_curse_anim) } else { (&mut ctx.skill_curse_e_model, &ctx.skill_curse_e_anim) };

    maanim_execute(model, Some(anim), frame, 0)?;
    draw_model(draw_context(&mut ctx.draw)?, model, x, y);

    Ok(())
}
