use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{
    AppContext, Texture, dialog_add_segment, get_design_height2, get_drawable_width, max_i32,
    min_i32, string_split, string_to_int,
};

pub type DialogEventHandler = fn(&mut AppContext, u64, i32, i32) -> Result<(), Fault>;
pub type DialogDrawHandler = fn(&mut AppContext) -> Result<(), Fault>;
pub type DialogUpdateHandler = fn(&mut AppContext, u64) -> Result<(), Fault>;

#[derive(Clone, Default)]
pub struct DialogSegment {
    pub x: i32,
    pub color: [i32; 3],
    pub flash: u8,
    pub texture: Texture,
}

#[derive(Clone, Default)]
pub struct DialogLine {
    pub y: i32,
    pub width: i32,
    pub scale: f32,
    pub segments: Vec<DialogSegment>,
}

#[derive(Clone, Default)]
pub struct Dialog {
    pub kind: i32,
    pub x: i32,
    pub y: i32,
    pub flags: i32,
    pub width: i32,
    pub height: i32,
    pub lines: Vec<DialogLine>,
    pub on_event: Option<DialogEventHandler>,
    pub state: i32,
    pub frame: i32,
    pub tick: i32,
    pub timer: i32,
    pub button_widths: [i32; 3],
    pub button_heights: [i32; 3],
    pub hover: [u8; 3],
    pub lit: [u8; 3],
    pub visible: [u8; 3],
    pub dimmable: [u8; 3],
    pub greyed: [u8; 3],
    pub audible: [u8; 3],
    pub close_hover: u8,
    pub button: i32,
    pub back_button: i32,
    pub on_update: Option<DialogUpdateHandler>,
    pub on_draw: Option<DialogDrawHandler>,
}

#[derive(Clone, Default)]
pub struct DialogManager {
    pub next_id: u64,
    pub objects: BTreeMap<u64, Dialog>,
    pub active: Vec<u64>,
    pub queued: Vec<u64>,
}

pub trait UiHost {
    fn message_set(&mut self, layer: i32, text: &[u8], size: i32, width: i32);
    fn message_clear(&mut self, layer: i32);
    fn option_window_build(&mut self, kind: i32);
    fn option_window_draw(&mut self);
    fn title_option_window_set_touchable(&mut self, touchable: u8);
    fn set_window_ratio(&mut self, ratio: f32);
    fn clear_layout_latch(&mut self);
    fn set_layout_latch(&mut self);
    fn viewport_resized(&mut self);
    fn page_list_layout(
        &mut self,
        list: usize,
        count: i32,
        rows: i32,
        center: f32,
        spacing: f32,
        width: f32,
    );
}

const TAG_LENGTHS: [i64; 5] = [4, 7, 8, 7, 8];

pub fn dialog_new(
    ctx: &mut AppContext,
    kind: i32,
    text: &[u8],
    x: i32,
    y: i32,
    flags: i32,
    on_event: Option<DialogEventHandler>,
) -> Result<u64, Fault> {
    ctx.dialogs.next_id = ctx.dialogs.next_id.wrapping_add(1);

    let dialog = ctx.dialogs.next_id;

    ctx.dialogs.objects.insert(
        dialog,
        Dialog {
            kind,
            x,
            y,
            flags,
            on_event,
            ..Dialog::default()
        },
    );

    if !text.is_empty() {
        let mut new_line = true;
        let mut red = 0xffi32;
        let mut green = 0xffi32;
        let mut blue = 0xffi32;
        let mut flash = 0u8;
        let mut pos = 0i64;

        loop {
            if new_line {
                let this = ctx
                    .dialogs
                    .objects
                    .get_mut(&dialog)
                    .ok_or(Fault::null_pointer())?;
                let count = this.lines.len() as u32;

                this.lines.push(DialogLine {
                    y: count.wrapping_add(1).wrapping_mul(0x24).wrapping_sub(0x24) as i32,
                    ..DialogLine::default()
                });
            }

            new_line = false;

            if (text.len() as u64) < pos as u64 {
                return Err(Fault::out_of_range());
            }

            let rest = &text[pos as usize..];
            let find = |tag: &[u8]| -> i64 {
                rest.windows(tag.len())
                    .position(|window| window == tag)
                    .map_or(-1, |at| at as i64 + pos)
            };
            let at_break = find(b"<br>");
            let at_color = find(b"<color=");
            let at_color_end = find(b"</color>");
            let at_flash = find(b"<flash>");
            let at_flash_end = find(b"</flash>");

            if at_break == -1
                && at_color == -1
                && at_color_end == -1
                && at_flash == -1
                && at_flash_end == -1
            {
                dialog_add_segment(ctx, dialog, rest, red, green, blue, flash)?;

                break;
            }

            let mut best = at_break;
            let mut tag = if at_break == -1 { -1i32 } else { 0 };

            if (at_color as u64) < best as u64 {
                tag = 1;
                best = at_color;
            }

            if at_color_end != -1 && (at_color_end as u64) < best as u64 {
                tag = 2;
                best = at_color_end;
            }

            if at_flash != -1 && (at_flash as u64) < best as u64 {
                tag = 3;
                best = at_flash;
            }

            if at_flash_end != -1 && (at_flash_end as u64) < best as u64 {
                tag = 4;
                best = at_flash_end;
            }

            if best != pos {
                let piece = &text[pos as usize..best as usize];

                dialog_add_segment(ctx, dialog, piece, red, green, blue, flash)?;
            }

            let next = match tag {
                0 => {
                    new_line = true;

                    (best as i32).wrapping_add(TAG_LENGTHS[0] as i32)
                }
                1 => {
                    let close = rest
                        .iter()
                        .position(|byte| *byte == b'>')
                        .map_or(-1, |at| at as i64 + pos);
                    let from = best.wrapping_add(TAG_LENGTHS[1]);

                    if (text.len() as u64) < from as u64 {
                        return Err(Fault::out_of_range());
                    }

                    let span = (close.wrapping_sub(from) as u64)
                        .min(text.len() as u64 - from as u64);
                    let cells =
                        string_split(&text[from as usize..from as usize + span as usize], b",");

                    red = string_to_int(cells.first().ok_or(Fault::null_pointer())?)?;
                    green = string_to_int(cells.get(1).ok_or(Fault::null_pointer())?)?;
                    blue = string_to_int(cells.get(2).ok_or(Fault::null_pointer())?)?;

                    (close as i32).wrapping_add(1)
                }
                2 => {
                    blue = 0xff;
                    green = 0xff;
                    red = 0xff;

                    (best as i32).wrapping_add(TAG_LENGTHS[2] as i32)
                }
                3 => {
                    flash = 1;

                    (best as i32).wrapping_add(TAG_LENGTHS[3] as i32)
                }
                4 => {
                    flash = 0;

                    (best as i32).wrapping_add(TAG_LENGTHS[4] as i32)
                }
                _ => best as i32,
            };

            pos = next as i64;

            if text.len() as u64 <= pos as u64 {
                break;
            }
        }
    }

    let drawable = get_drawable_width(ctx)?;
    let design = get_design_height2(ctx);
    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let pad = ctx.i32_at(AppContext::LETTERBOX_PAD)?;
    let this = ctx
        .dialogs
        .objects
        .get_mut(&dialog)
        .ok_or(Fault::null_pointer())?;
    let mut widest = 0i32;

    for line in this.lines.iter_mut() {
        widest = min_i32(max_i32(widest, line.width), 0x320);
        line.scale = 1.0;
    }

    for line in this.lines.iter_mut() {
        if line.width > widest {
            line.scale = widest as f32 / line.width as f32;
            line.width = widest;
        }
    }

    this.width = max_i32(widest.wrapping_add(0x36), 0x29e);

    let count = this.lines.len() as u64;

    this.height = if count >= 3 {
        ((count as u32).wrapping_mul(9) << 2).wrapping_add(0x48) as i32
    } else {
        0xb4i32.wrapping_sub(((count as i32) ^ 3).wrapping_mul(0x12))
    };

    if flags & 2 == 0 {
        this.x = ops::div_2(drawable)
            .wrapping_add(this.x)
            .wrapping_sub(ops::div_2(this.width));

        if flags & 1 == 0 {
            this.y = this.y.wrapping_sub(shift);
            this.y = this.y.wrapping_sub(pad);
            this.y = ops::div_2(design)
                .wrapping_add(this.y)
                .wrapping_sub(ops::div_2(this.height));

            if (this.kind.wrapping_sub(1) as u32) <= 2 {
                this.y = this.y.wrapping_add(-0x28);
            }
        }
    }

    this.state = 0;
    this.frame = 0;
    this.tick = 0;
    this.timer = 0;
    this.button_widths = [0xa8; 3];
    this.button_heights = [0x48; 3];
    this.audible = [1; 3];
    this.visible = [1; 3];
    this.lit = [1; 3];
    this.dimmable = [1; 3];
    this.greyed = [0; 3];
    this.hover = [0; 3];
    this.close_hover = 0;
    this.button = -1;
    this.back_button = -3;
    this.on_update = None;
    this.on_draw = None;

    Ok(dialog)
}
