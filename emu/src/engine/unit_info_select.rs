use crate::Fault;

use super::{get_button_unit_form, get_button_unit_id, get_cat_name, get_text_texture, load_conjure_desc_textures, text_texture_cache, AppContext};

pub fn unit_info_select(ctx: &mut AppContext, slot: i32) -> Result<(), Fault> {
    let unit_id = get_button_unit_id(ctx, 0, slot)?;
    let form = get_button_unit_form(ctx, 0, slot)?;
    let name = get_cat_name(ctx, unit_id, form);
    let font = ctx.default_font.clone();
    let texture = get_text_texture(text_texture_cache(ctx)?, &name, &font, 0x1e, 1, 0);

    ctx.unit_info_texts[0] = Some(texture);
    ctx.unit_info_texts[1] = None;
    ctx.unit_info_texts[2] = None;
    ctx.unit_info_texts[3] = None;

    load_conjure_desc_textures(ctx, 1, unit_id, 0)?;

    Ok(())
}
