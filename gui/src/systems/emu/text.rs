use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use emu::engine::{FormatArg, TextRenderer, Texture};
use kore::common::assets::{
    FONT_JP_BOLD, FONT_KR_BOLD, FONT_LATIN_BOLD, FONT_SYMBOLS, FONT_TC_BOLD, FONT_TH_BOLD,
};

use super::assets::{Sheet, SheetCache, WIDE_COMMA};

const ALIGN_CENTER: i32 = 1;
const ALIGN_RIGHT: i32 = 2;
const LAYOUT_PAD: u32 = 4;

static NEXT_LABEL: AtomicU64 = AtomicU64::new(1);

type LabelKey = (Box<[u8]>, i32, i32, i32);

pub struct Formatter {
    sheets: Rc<RefCell<SheetCache>>,
    labels: BTreeMap<LabelKey, Texture>,
}

pub const LABEL_PREFIX: &str = "label:";

pub fn label_key(id: u64) -> Box<str> {
    Box::from(format!("{LABEL_PREFIX}{id}").as_str())
}

struct Line {
    glyphs: Vec<(usize, ab_glyph::GlyphId, f32)>,
    width: f32,
    ascent: f32,
    descent: f32,
}

impl Line {
    fn empty(faces: &[FontRef<'_>], size: f32) -> Self {
        let lead = faces.first().map(|face| face.as_scaled(Formatter::em(face, size)));

        Self {
            glyphs: Vec::new(),
            width: 0.0,
            ascent: lead.as_ref().map_or(0.0, ScaleFont::ascent),
            descent: lead.as_ref().map_or(0.0, |face| -face.descent()),
        }
    }
}

impl Formatter {
    pub fn new(sheets: Rc<RefCell<SheetCache>>) -> Self {
        Self {
            sheets,
            labels: BTreeMap::new(),
        }
    }

    fn faces() -> Vec<FontRef<'static>> {
        [
            FONT_LATIN_BOLD,
            FONT_JP_BOLD,
            FONT_KR_BOLD,
            FONT_TC_BOLD,
            FONT_TH_BOLD,
            FONT_SYMBOLS,
        ]
        .into_iter()
        .filter_map(|bytes| FontRef::try_from_slice(bytes).ok())
        .collect()
    }

    fn em(face: &FontRef<'_>, size: f32) -> PxScale {
        let height = face.height_unscaled();

        PxScale::from(face.units_per_em().map_or(size, |units| size * height / units))
    }

    fn lines(faces: &[FontRef<'_>], text: &str, size: f32, wrap: f32) -> Vec<Line> {
        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            let mut line = Line::empty(faces, size);
            let mut last: Option<(usize, ab_glyph::GlyphId)> = None;

            for letter in paragraph.chars() {
                let Some((face, id)) = faces
                    .iter()
                    .enumerate()
                    .map(|(face, font)| (face, font.glyph_id(letter)))
                    .find(|(_, id)| id.0 != 0)
                    .or_else(|| faces.first().map(|font| (0, font.glyph_id(letter))))
                else {
                    continue;
                };
                let scaled = faces[face].as_scaled(Self::em(&faces[face], size));
                let advance = scaled.h_advance(id);
                let pair = |before: Option<(usize, ab_glyph::GlyphId)>| {
                    before
                        .filter(|(held, _)| *held == face)
                        .map_or(0.0, |(_, left)| scaled.kern(left, id))
                };

                if wrap > 0.0 && line.width + pair(last) + advance > wrap && !line.glyphs.is_empty() {
                    lines.push(std::mem::replace(&mut line, Line::empty(faces, size)));
                    last = None;
                }

                let kern = pair(last);

                line.glyphs.push((face, id, line.width + kern));
                line.width += kern + advance;
                line.ascent = line.ascent.max(scaled.ascent());
                line.descent = line.descent.max(-scaled.descent());
                last = Some((face, id));
            }

            lines.push(line);
        }

        lines
    }

    fn rasterize(text: &str, size: i32, align: i32, wrap: i32) -> Option<Sheet> {
        let faces = Self::faces();
        let lines = Self::lines(&faces, text, size as f32, wrap as f32);
        let widest = lines.iter().map(|line| line.width).fold(0.0, f32::max);
        let width = if wrap > 0 {
            wrap as u32
        } else {
            (widest + 1.0).floor() as u32 + LAYOUT_PAD
        }
        .max(1);
        let height = lines
            .iter()
            .map(|line| (line.ascent + line.descent).ceil())
            .sum::<f32>()
            .max(1.0) as u32;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let mut top = 0.0f32;

        for line in &lines {
            let slack = width as f32 - line.width;
            let left = match align {
                ALIGN_CENTER => slack / 2.0,
                ALIGN_RIGHT => slack,
                _ => 0.0,
            };
            let baseline = top + line.ascent;

            top += (line.ascent + line.descent).ceil();

            for (face, id, x) in &line.glyphs {
                let Some(font) = faces.get(*face) else {
                    continue;
                };
                let glyph = id.with_scale_and_position(Self::em(font, size as f32), point(left + x, baseline));
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

        Some(Sheet::new(width, height, pixels.as_slice()))
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

        let shown = String::from_utf8_lossy(text).replace(WIDE_COMMA, ",");
        let Some(sheet) = Self::rasterize(&shown, size, align, width) else {
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

            let marker = [b"${".as_slice(), name, b"}".as_slice()].concat();
            let name = marker.as_slice();
            let mut at = 0usize;

            while at + name.len() <= out.len() {
                if &out[at..at + name.len()] == name {
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
        Self::lines(
            &Self::faces(),
            &String::from_utf8_lossy(text).replace(WIDE_COMMA, ","),
            size as f32,
            0.0,
        )
        .iter()
        .map(|line| line.width)
        .fold(0.0, f32::max)
        .ceil() as i32
    }
}
