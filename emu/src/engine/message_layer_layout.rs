use crate::{Fault, ops};

use super::{
    AppContext, TextBlock, TextLine, get_drawable_width, max_i32, message_layer_push_run, min_i32,
    string_split, string_to_int,
};

const MARKERS: [&[u8]; 5] = [b"<br>", b"<color=", b"</color>", b"<flash>", b"</flash>"];
const MARKER_LENGTHS: [usize; 5] = [4, 7, 8, 7, 8];
const BREAK: usize = 0;
const COLOR: usize = 1;
const COLOR_END: usize = 2;
const FLASH: usize = 3;
const FLASH_END: usize = 4;
const LINE_HEIGHT: f64 = 1.2;
const WHITE: i32 = 0xff;
const CHANNELS: usize = 3;
const NOT_FOUND: usize = usize::MAX;

pub fn message_layer_layout(
    ctx: &mut AppContext,
    block: &mut TextBlock,
    text: &[u8],
    size: i32,
    width: i32,
) -> Result<(), Fault> {
    let width = if width == 0 {
        get_drawable_width(ctx)?
    } else {
        width
    };

    block.width = width;
    block.spacing = size;
    block.blink = 0;

    if !text.is_empty() {
        let mut cursor = 0usize;
        let mut opening = true;
        let mut blink = 0u8;
        let mut red = WHITE;
        let mut green = WHITE;
        let mut blue = WHITE;

        loop {
            if opening {
                let index = block.lines.len() as i32;

                block.lines.push(TextLine {
                    y: ops::cvttsd2si(size as f64 * LINE_HEIGHT * index as f64),
                    width: 0,
                    scale: 0.0,
                    glyphs: Vec::new(),
                });

                opening = false;
            }

            let rest = text.get(cursor..).ok_or(Fault::out_of_range())?;
            let mut found: Option<(usize, usize)> = None;

            for (marker, needle) in MARKERS.iter().enumerate() {
                let Some(offset) = rest.windows(needle.len()).position(|run| run == *needle) else {
                    continue;
                };
                let at = cursor.wrapping_add(offset);

                if found.is_none_or(|(held, _)| at < held) {
                    found = Some((at, marker));
                }
            }

            let Some((at, marker)) = found else {
                let run = text.get(cursor..).ok_or(Fault::out_of_range())?.to_vec();

                message_layer_push_run(ctx, block, &run, red, green, blue, blink, size)?;

                break;
            };

            if at != cursor {
                let run = text.get(cursor..at).ok_or(Fault::out_of_range())?.to_vec();

                message_layer_push_run(ctx, block, &run, red, green, blue, blink, size)?;
            }

            let scanned = cursor;

            cursor = at.wrapping_add(MARKER_LENGTHS[marker]);

            match marker {
                BREAK => opening = true,
                FLASH => blink = 1,
                FLASH_END => blink = 0,
                COLOR_END => {
                    red = WHITE;
                    green = WHITE;
                    blue = WHITE;
                }
                COLOR => {
                    let mut close = NOT_FOUND;

                    if text.len() >= scanned {
                        let rest = text.get(scanned..).ok_or(Fault::out_of_range())?;

                        if let Some(offset) = rest.iter().position(|byte| *byte == b'>') {
                            close = scanned.wrapping_add(offset);
                        }
                    }

                    if text.len() < cursor {
                        return Err(Fault::out_of_range());
                    }

                    let mut span = close.wrapping_sub(cursor);
                    let remaining = text.len().wrapping_sub(cursor);

                    if remaining < span {
                        span = remaining;
                    }

                    let channels = string_split(
                        text.get(cursor..cursor.wrapping_add(span))
                            .ok_or(Fault::out_of_range())?,
                        b",",
                    );

                    for (channel, value) in channels.iter().enumerate().take(CHANNELS) {
                        let parsed = string_to_int(value)?;

                        match channel {
                            0 => red = parsed,
                            1 => green = parsed,
                            _ => blue = parsed,
                        }
                    }

                    cursor = close.wrapping_add(1);
                }
                _ => {}
            }

            if text.len() <= cursor {
                break;
            }
        }
    }

    let mut widest = 0i32;

    for line in &mut block.lines {
        widest = min_i32(max_i32(widest, line.width), block.width);
        line.scale = 1.0;
    }

    for line in &mut block.lines {
        if line.width > widest {
            line.scale = widest as f32 / line.width as f32;
            line.width = widest;
        }
    }

    Ok(())
}
