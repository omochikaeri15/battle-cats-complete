use crate::Fault;

use super::{AppContext, img_dialog_context_init, texture_cache_load};

pub fn texture_context_init(ctx: &mut AppContext) -> Result<(), Fault> {
    img_dialog_context_init(ctx)?;
    ctx.img039_sheet = None;
    ctx.img039_sheet = texture_cache_load(ctx, b"img039.png", b"img039.imgcut", 0x2601)?;

    Ok(())
}
