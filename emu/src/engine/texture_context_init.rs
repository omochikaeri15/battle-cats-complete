use crate::Fault;

use super::{texture_cache_load, AppContext};

pub fn texture_context_init(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.scene_host().ok_or(Fault::HostMissing { site: "texture_context_init" })?.scene_base_init();
    ctx.img039_sheet = None;
    ctx.img039_sheet = texture_cache_load(ctx, b"img039.png", b"img039.imgcut", 0x2601)?;

    Ok(())
}
