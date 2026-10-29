use crate::Fault;

use super::{
    AppContext, Surface, TextBlock, TextGlyph, get_text_texture, text_texture_cache,
    texture_get_width,
};

pub fn message_layer_push_run(
    ctx: &mut AppContext,
    block: &mut TextBlock,
    text: &[u8],
    red: i32,
    green: i32,
    blue: i32,
    blink: u8,
    size: i32,
) -> Result<(), Fault> {
    let font = ctx.default_font.clone();
    let texture = get_text_texture(text_texture_cache(ctx)?, text, &font, size, 0, 0);
    let added = texture_get_width(Surface::Label(&texture));
    let line = block.lines.last_mut().ok_or(Fault::index_out_of_range(-1, 0))?;

    line.glyphs.push(TextGlyph {
        x: line.width,
        red,
        green,
        blue,
        blink,
        texture: Some(texture),
    });

    line.width = line.width.wrapping_add(added);

    Ok(())
}
