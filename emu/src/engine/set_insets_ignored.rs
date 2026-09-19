use crate::Fault;

use super::{set_draw_origin, AppContext};

pub fn set_insets_ignored(ctx: &mut AppContext, ignored: u8) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::INSETS_IGNORED, [ignored])?;

    let origin = ctx
        .i32_at(AppContext::LETTERBOX_PAD)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    set_draw_origin(ctx, 0, origin)
}
