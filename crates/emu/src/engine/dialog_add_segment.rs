use crate::Fault;

use super::{
    AppContext, DialogSegment, Surface, get_text_texture, text_texture_cache, texture_get_width,
};

pub fn dialog_add_segment(
    ctx: &mut AppContext,
    dialog: u64,
    text: &[u8],
    red: i32,
    green: i32,
    blue: i32,
    flash: u8,
) -> Result<(), Fault> {
    let font = ctx.default_font.clone();
    let texture = get_text_texture(text_texture_cache(ctx)?, text, &font, 0x1e, 0, 0);
    let line = ctx
        .dialogs
        .objects
        .get_mut(&dialog)
        .and_then(|this| this.lines.last_mut())
        .ok_or(Fault::null_pointer())?;

    line.segments.push(DialogSegment::default());

    let x = line.width;

    line.width = line.width.wrapping_add(texture_get_width(Surface::Label(&texture)));

    let segment = line.segments.last_mut().ok_or(Fault::null_pointer())?;

    segment.texture = texture;
    segment.x = x;
    segment.color = [red, green, blue];
    segment.flash = flash;

    Ok(())
}
