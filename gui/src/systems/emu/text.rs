use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use emu::engine::{FormatArg, TextRenderer, Texture};
use kore::common::assets::FONT_JP;

use super::assets::{Sheet, SheetCache};

const ALIGN_CENTER: i32 = 1;
const ALIGN_RIGHT: i32 = 2;

static NEXT_LABEL: AtomicU64 = AtomicU64::new(1);

type LabelKey = (Box<[u8]>, i32, i32, i32);

pub struct Formatter {
    sheets: Rc<RefCell<SheetCache>>,
    labels: BTreeMap<LabelKey, Texture>,
}

pub fn label_key(id: u64) -> Box<str> {
    Box::from(format!("label:{id}").as_str())
}

struct Line {
    glyphs: Vec<(ab_glyph::GlyphId, f32)>,
    width: f32,
}

impl Formatter {
    pub fn new(sheets: Rc<RefCell<SheetCache>>) -> Self {
        Self {
            sheets,
            labels: BTreeMap::new(),
        }
    }

    fn lines(font: &FontRef<'_>, text: &str, size: f32, wrap: f32) -> Vec<Line> {
        let scaled = font.as_scaled(PxScale::from(size));
        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            let mut line = Line {
                glyphs: Vec::new(),
                width: 0.0,
            };
            let mut last = None;

            for letter in paragraph.chars() {
                let id = font.glyph_id(letter);
                let kern = last.map_or(0.0, |before| scaled.kern(before, id));
                let advance = scaled.h_advance(id);

                if wrap > 0.0 && line.width + kern + advance > wrap && !line.glyphs.is_empty() {
                    lines.push(std::mem::replace(
                        &mut line,
                        Line {
                            glyphs: Vec::new(),
                            width: 0.0,
                        },
                    ));
                    last = None;
                }

                let kern = last.map_or(0.0, |before| scaled.kern(before, id));

                line.glyphs.push((id, line.width + kern));
                line.width += kern + advance;
                last = Some(id);
            }

            lines.push(line);
        }

        lines
    }

    fn rasterize(text: &str, size: i32, align: i32, wrap: i32) -> Option<Sheet> {
        let font = FontRef::try_from_slice(FONT_JP).ok()?;
        let scale = PxScale::from(size as f32);
        let scaled = font.as_scaled(scale);
        let lines = Self::lines(&font, text, size as f32, wrap as f32);
        let widest = lines.iter().map(|line| line.width).fold(0.0, f32::max);
        let pitch = scaled.height() + scaled.line_gap();
        let width = if wrap > 0 { wrap as u32 } else { widest.ceil() as u32 }.max(1);
        let height = (pitch * lines.len() as f32).ceil().max(1.0) as u32;
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        for (row, line) in lines.iter().enumerate() {
            let slack = width as f32 - line.width;
            let left = match align {
                ALIGN_CENTER => slack / 2.0,
                ALIGN_RIGHT => slack,
                _ => 0.0,
            };
            let baseline = pitch * row as f32 + scaled.ascent();

            for (id, x) in &line.glyphs {
                let glyph = id.with_scale_and_position(scale, point(left + x, baseline));
                let Some(outline) = font.outline_glyph(glyph) else {
                    continue;
                };
                let bounds = outline.px_bounds();

                outline.draw(|dx, dy, coverage| {
                    let px = bounds.min.x as i32 + dx as i32;
                    let py = bounds.min.y as i32 + dy as i32;

                    if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                        return;
                    }

                    let at = ((py as u32 * width + px as u32) * 4) as usize;
                    let shade = (coverage.clamp(0.0, 1.0) * 255.0) as u8;

                    if let Some(pixel) = pixels.get_mut(at..at + 4) {
                        let held = pixel[3].max(shade);

                        pixel.copy_from_slice(&[held; 4]);
                    }
                });
            }
        }

        Some(Sheet {
            width,
            height,
            pixels: Arc::from(pixels.as_slice()),
        })
    }
    fn expand(pattern: &[u8], args: &[FormatArg<'_>]) -> Vec<u8> {
        let mut out = Vec::with_capacity(pattern.len());
        let mut next = 0usize;
        let mut at = 0usize;

        while at < pattern.len() {
            if pattern[at] != b'%' {
                out.push(pattern[at]);
                at += 1;

                continue;
            }

            let mut scan = at + 1;

            if pattern.get(scan) == Some(&b'%') {
                out.push(b'%');
                at = scan + 1;

                continue;
            }

            let zero = pattern.get(scan) == Some(&b'0');

            if zero {
                scan += 1;
            }

            let mut width = 0usize;

            while pattern.get(scan).is_some_and(u8::is_ascii_digit) {
                width = width * 10 + (pattern[scan] - b'0') as usize;
                scan += 1;
            }

            let Some(kind) = pattern.get(scan).copied() else {
                out.push(pattern[at]);
                at += 1;

                continue;
            };

            match (kind, args.get(next)) {
                (b'd', Some(FormatArg::Int(value))) => {
                    let digits = value.unsigned_abs().to_string();
                    let sign = usize::from(*value < 0);
                    let pad = width.saturating_sub(digits.len() + sign);

                    if *value < 0 {
                        out.push(b'-');
                    }

                    out.extend(std::iter::repeat_n(if zero { b'0' } else { b' ' }, pad));
                    out.extend_from_slice(digits.as_bytes());
                    next += 1;
                }
                (b'd', Some(FormatArg::Text(text))) | (b's' | b'@', Some(FormatArg::Text(text))) => {
                    out.extend_from_slice(text);
                    next += 1;
                }
                (b's' | b'@', Some(FormatArg::Int(value))) => {
                    out.extend_from_slice(value.to_string().as_bytes());
                    next += 1;
                }
                _ => out.extend_from_slice(&pattern[at..=scan]),
            }

            at = scan + 1;
        }

        out
    }
}

impl TextRenderer for Formatter {
    fn text_texture(
        &mut self,
        text: &[u8],
        _font: &[u8],
        size: i32,
        align: i32,
        width: i32,
    ) -> Texture {
        let key: LabelKey = (Box::from(text), size, align, width);

        if let Some(texture) = self.labels.get(&key) {
            return *texture;
        }

        let Some(sheet) = Self::rasterize(&String::from_utf8_lossy(text), size, align, width) else {
            return Texture::default();
        };
        let texture = Texture {
            id: NEXT_LABEL.fetch_add(1, Ordering::Relaxed),
            width: sheet.width as i32,
            height: sheet.height as i32,
        };

        self.sheets.borrow_mut().insert(label_key(texture.id), sheet);
        self.labels.insert(key, texture);

        texture
    }

    fn format(&mut self, pattern: &[u8], args: &[&[u8]]) -> Vec<u8> {
        let owned: Vec<FormatArg<'_>> = args.iter().map(|text| FormatArg::Text(text)).collect();

        Self::expand(pattern, &owned)
    }

    fn substitute(&mut self, text: &[u8], tokens: &[(&[u8], &[u8])]) -> Vec<u8> {
        let mut out = text.to_vec();

        for (name, value) in tokens {
            if name.is_empty() {
                continue;
            }

            let mut at = 0usize;

            while at + name.len() <= out.len() {
                if &out[at..at + name.len()] == *name {
                    out.splice(at..at + name.len(), value.iter().copied());
                    at += value.len();
                } else {
                    at += 1;
                }
            }
        }

        out
    }

    fn format_args(&mut self, pattern: &[u8], args: &[FormatArg<'_>]) -> Vec<u8> {
        Self::expand(pattern, args)
    }

    fn stage_name(&mut self, _map_type: i32, _map_index: i32, _stage: i32) -> Vec<u8> {
        Vec::new()
    }

    fn text_width(&mut self, text: &[u8], size: i32) -> i32 {
        FontRef::try_from_slice(FONT_JP).map_or(0, |font| {
            Self::lines(&font, &String::from_utf8_lossy(text), size as f32, 0.0)
                .iter()
                .map(|line| line.width)
                .fold(0.0, f32::max)
                .ceil() as i32
        })
    }
}
