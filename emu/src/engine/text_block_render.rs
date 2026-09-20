use crate::{Fault, operation};

use super::{
    DrawSink, Surface, draw_context, draw_surface_scaled, set_tint, texture_get_height,
    texture_get_width,
};

#[derive(Clone, Default, Debug)]
pub struct TextGlyph {
    pub x: i32,
    pub red: i32,
    pub green: i32,
    pub blue: i32,
    pub blink: u8,
    pub texture: Option<super::Texture>,
}

#[derive(Clone, Default, Debug)]
pub struct TextLine {
    pub y: i32,
    pub width: i32,
    pub scale: f32,
    pub glyphs: Vec<TextGlyph>,
}

#[derive(Clone, Default, Debug)]
pub struct TextBlock {
    pub lines: Vec<TextLine>,
    pub blink: i32,
    pub spacing: i32,
}

pub fn text_block_render(
    sink: &mut Option<Box<dyn DrawSink>>,
    block: &mut TextBlock,
    x: i32,
    y: i32,
    align: i32,
    scale: f32,
) -> Result<(), Fault> {
    let blink = block.blink;

    block.blink = if blink < 4 { blink.wrapping_add(1) } else { 0 };

    let left = x as f32;

    for line in &block.lines {
        for glyph in &line.glyphs {
            if glyph.blink != 0 {
                let phase = block.blink % 4;
                let dc = draw_context(sink)?;

                if phase > 1 {
                    set_tint(dc, 0xff, 0, 0xff, 0xff);
                } else {
                    set_tint(dc, 0xff, 0xff, 0, 0xff);
                }
            } else {
                set_tint(
                    draw_context(sink)?,
                    glyph.red,
                    glyph.green,
                    glyph.blue,
                    0xff,
                );
            }

            let shift = if align & 1 != 0 {
                operation::cvttss2si(line.width as f32 * scale * 0.5)
            } else if align & 2 != 0 {
                operation::cvttss2si(line.width as f32 * scale)
            } else {
                0
            };

            let lift = if align & 4 != 0 {
                let last = block.lines.last().ok_or(Fault::index_out_of_range(-1, 0))?;
                let lift =
                    operation::cvttss2si(last.y.wrapping_add(block.spacing) as f32 * scale * 0.5);

                if block.lines.len() & 1 != 0 {
                    lift
                } else {
                    operation::cvttsd2si((block.spacing as f32 * scale) as f64 * 0.1 + lift as f64)
                }
            } else if align & 8 != 0 {
                let last = block.lines.last().ok_or(Fault::index_out_of_range(-1, 0))?;

                operation::cvttss2si(last.y.wrapping_add(block.spacing) as f32 * scale)
            } else {
                0
            };

            let dc = draw_context(sink)?;
            let across = operation::cvttss2si(glyph.x as f32 * line.scale + left - shift as f32);
            let down = y.wrapping_sub(lift).wrapping_add(line.y);
            let texture = glyph
                .texture
                .as_ref()
                .ok_or(Fault::null_pointer())?;
            let width = operation::cvttss2si(
                texture_get_width(Surface::Label(texture)) as f32 * line.scale * scale,
            );
            let height =
                operation::cvttss2si(texture_get_height(Surface::Label(texture)) as f32 * scale);

            draw_surface_scaled(dc, Surface::Label(texture), across, down, width, height);
        }
    }

    Ok(())
}
