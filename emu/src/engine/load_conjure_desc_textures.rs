use crate::Fault;

use super::{
    AppContext, get_text_texture, get_unit_form, stat_conjure_unit_id, text_texture_cache,
};

pub fn load_conjure_desc_textures(
    ctx: &mut AppContext,
    first_slot: usize,
    unit_id: i32,
    index: i32,
) -> Result<i32, Fault> {
    let form = get_unit_form(ctx, unit_id)?;
    let spirit = stat_conjure_unit_id(ctx, 0, unit_id, form)?;

    if spirit < 0 {
        return Ok(-1);
    }

    let font = ctx.default_font.clone();

    for line in 0..3usize {
        let text = ctx
            .cat_names
            .get(spirit as usize)
            .map(|forms| forms[0][line + 1].clone())
            .unwrap_or_default();

        match text.len() {
            0 => break,
            1 if text[0] == b' ' => break,
            3 if text == b"\xe3\x80\x80" => break,
            _ => {}
        }

        let texture = get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0);

        if let Some(slot) = ctx
            .unit_info_texts
            .get_mut(first_slot.wrapping_add((index as i64 as usize).wrapping_add(line)))
        {
            *slot = Some(texture);
        }
    }

    Ok(index)
}
