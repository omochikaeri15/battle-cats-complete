use crate::{operation, Fault};

use super::{draw_context, draw_model, get_drawable_width, maanim_execute, AppContext};

pub fn draw_demon_battle_banner(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.i32_at(AppContext::DEMON_BANNER_FRAME)? < 0x1e {
        return Ok(());
    }

    let frame = ctx.i32_at(AppContext::DEMON_BANNER_FRAME)?.wrapping_add(-0x1e);

    maanim_execute(&mut ctx.demonbattle_model, Some(&ctx.demon_banner_anim), frame, 0)?;

    let x = operation::div_2(get_drawable_width(ctx)?);

    draw_model(draw_context(&mut ctx.draw)?, &ctx.demonbattle_model, x, 0x96);

    Ok(())
}
