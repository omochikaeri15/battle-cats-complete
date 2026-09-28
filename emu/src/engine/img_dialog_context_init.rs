use crate::Fault;

use super::{
    AppContext, Texture, get_scene_id, get_text_texture, query_localizable, scene_img008_load,
    text_texture_cache, texture_cache_load,
};

pub fn img_dialog_context_init(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.scene_img005_sheet = None;
    ctx.dialog_sheet = None;
    ctx.scene_img008_sheet = None;
    ctx.guide_label = Texture::default();
    ctx.scene_img005_sheet = texture_cache_load(ctx, b"img005.png", b"img005.imgcut", 0x2601)?;
    ctx.dialog_sheet = texture_cache_load(ctx, b"img005_1.png", b"img005_1.imgcut", 0x2601)?;

    if get_scene_id(ctx)? != 0x61 {
        ctx.scene_img008_sheet = scene_img008_load(ctx)?;
    }

    let text = query_localizable(ctx, b"unlockpopup_Guide_1");
    let font = ctx.default_font.clone();

    ctx.guide_label = get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x14, 0, 0);

    Ok(())
}
