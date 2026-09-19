use crate::Fault;

use super::{draw_context, get_left_inset_logical, has_insets, scene_ignores_insets, AppContext};

pub fn set_draw_origin(ctx: &mut AppContext, x: i32, y: i32) -> Result<(), Fault> {
    let x = if has_insets(ctx) && scene_ignores_insets(ctx)? == 0 { x.wrapping_add(get_left_inset_logical(ctx)) } else { x };

    draw_context(&mut ctx.draw)?.set_origin(x, y);

    Ok(())
}
