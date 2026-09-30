use std::borrow::Cow;
use std::sync::Arc;

use image::{self, RgbaImage};
use serde::Serialize;

use crate::common::columns::{self, Column};
use crate::common::stream::{cell_value, declared_rows, Reader};

use super::RigError;

const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const MINIMUM_PNG_LENGTH: usize = 33;

/// The alpha at or above which a pixel counts towards a cut's opaque extent.
const ALPHA_FLOOR: u8 = 8;

/// The most points a cut's visible hull may carry.
const HULL_CAP: usize = 64;

/// One sprite region within a texture atlas, in source pixels.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SpriteCut {
    /// The left edge of the region.
    pub x: i32,
    /// The top edge of the region.
    pub y: i32,
    /// The width of the region.
    pub width: i32,
    /// The height of the region.
    pub height: i32,
    /// The region's declared name, which is empty when the cut list supplies none.
    pub name: String,
}

impl SpriteCut {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of an `.imgcut` region row
    /// from the parser's own table instead of restating it. [`SpriteCut::name`]
    /// is the row's trailing text rather than a column.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        x      : 0;
        y      : 1;
        width  : 2;
        height : 3;
    };
}

/// The tightest rectangle within a cut that contains a visible pixel.
///
/// The atlas pads its cuts with transparency that draws nothing, so a consumer
/// measuring the area a part occupies wants this rather than the declared
/// region. The coordinates are in the same atlas pixel space as
/// [`SpriteCut`], so they can be compared with a cut's own edges directly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Opaque {
    /// The left edge of the visible region.
    pub x: i32,
    /// The top edge of the visible region.
    pub y: i32,
    /// The width of the visible region.
    pub width: i32,
    /// The height of the visible region.
    pub height: i32,
}

/// A decoded texture atlas and the sprite regions carved out of it.
#[derive(Clone, Debug, Default)]
pub struct SpriteSheet {
    /// The decoded atlas with its color channels premultiplied by alpha, shared cheaply between clones.
    pub image_data: Option<Arc<RgbaImage>>,
    /// The declared format version of the cut list.
    pub version: i32,
    /// The file name of the atlas the cut list names.
    pub image_name: String,
    /// The sprite regions carved out of the atlas, in the order the file declares them.
    pub cuts: Vec<SpriteCut>,
    /// The visible extent measured within each cut, parallel to `cuts`, holding `None` where a cut draws nothing at all.
    ///
    /// This is measured from the atlas pixels rather than declared by the cut
    /// list, so it is empty when the sheet carries no decoded image to measure.
    pub opaque: Vec<Option<Opaque>>,
    /// The convex hull of each cut's visible pixels, parallel to `cuts`, holding an empty hull where a cut draws nothing at all.
    ///
    /// [`Opaque`] is an axis-aligned rectangle, so its corners are transparent
    /// whenever the sprite inside it is not itself rectangular. Those corners
    /// cost nothing while the part is unrotated and inflate its measured extent
    /// as soon as it turns, so a consumer measuring a rotated part wants this
    /// instead. The points address pixel corners rather than pixel centres, so
    /// the right edge of a cut's visible pixels reads as `x + width` in the same
    /// atlas pixel space as [`SpriteCut`].
    ///
    /// A hull is held to sixty-four points by merging neighbouring columns
    /// before it is built, so a cut whose outline is finer than that reports a
    /// bound enclosing its visible pixels rather than their exact hull.
    pub hull: Vec<Vec<(i32, i32)>>,
}

impl SpriteSheet {
    /// Parses a texture atlas and its sprite cut list into a resolved sprite sheet.
    ///
    /// Color channels are premultiplied by alpha and fully transparent pixels
    /// zeroed. A texture that fails to decode is passed through salvage before
    /// being abandoned.
    ///
    /// The cut list's first three lines are discarded unread beyond the version
    /// and atlas name they carry, and exactly as many region rows are read as the
    /// fourth line declares, so trailing rows are ignored and a file that runs out
    /// repeats the last row it managed to read.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the PNG texture atlas.
    /// * `imgcut` - The raw bytes of the `.imgcut` sprite region list.
    ///
    /// # Returns
    /// A `Result` containing the resolved `SpriteSheet` on success, or a
    /// `RigError` if the atlas could not be decoded, the cut list described no
    /// usable regions, or it declared a count the engine could not allocate.
    pub fn parse(png: impl AsRef<[u8]>, imgcut: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8]) -> Result<Self, RigError> {
        let mut image_opt = image::load_from_memory(png).map(|img| img.to_rgba8()).ok();

        if image_opt.is_none() && png.starts_with(&PNG_MAGIC) {
            image_opt = Self::repair_inner(png).ok();
        }

        let mut image = image_opt.ok_or(RigError::ImageDecodeFailed)?;

        for pixel in image.pixels_mut() {
            let alpha = pixel[3] as u32;

            if alpha == 0 {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                continue;
            }

            if alpha < 255 {
                pixel[0] = ((pixel[0] as u32 * alpha) / 255) as u8;
                pixel[1] = ((pixel[1] as u32 * alpha) / 255) as u8;
                pixel[2] = ((pixel[2] as u32 * alpha) / 255) as u8;
            }
        }

        let content = String::from_utf8_lossy(imgcut);
        let mut reader = Reader::new(&content);

        reader.line();

        let version = reader.line().map_or(0, cell_value);
        let image_name = reader.line().map(|line| line.trim().to_string()).unwrap_or_default();

        reader.row();
        let count = declared_rows(reader.value(0)).ok_or(RigError::CountTooLarge)?;

        let mut cuts = Vec::with_capacity(count);

        for _ in 0..count {
            reader.row();

            let mut cut = SpriteCut::default();
            let trailing = columns::apply(&reader.scalars(), SpriteCut::COLUMNS, &mut cut);
            cut.name = reader.cells().get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            cuts.push(cut);
        }

        if cuts.is_empty() {
            return Err(RigError::NoSpriteCuts);
        }

        let (opaque, hull) = cuts
            .iter()
            .map(|cut| measure(&image, cut).map_or((None, Vec::new()), |(rect, hull)| (Some(rect), hull)))
            .unzip();

        Ok(Self { image_data: Some(Arc::new(image)), version, image_name, cuts, opaque, hull })
    }

    /// Returns the visible extent of one cut, falling back to the cut's own region.
    ///
    /// A sheet parsed without pixel data has nothing to measure, so every cut
    /// reports the region the cut list declares for it.
    ///
    /// # Arguments
    /// * `index` - The index of the cut in `cuts`.
    ///
    /// # Returns
    /// An `Option` containing the visible region, or `None` for a cut that draws
    /// nothing or an index no cut occupies.
    pub fn visible(&self, index: usize) -> Option<Opaque> {
        let cut = self.cuts.get(index)?;

        match self.opaque.get(index) {
            Some(measured) => *measured,
            None => Some(Opaque { x: cut.x, y: cut.y, width: cut.width, height: cut.height }),
        }
    }

    /// Returns the hull of one cut's visible pixels, falling back to the cut's own region.
    ///
    /// A sheet parsed without pixel data has nothing to measure, so every cut
    /// reports the four corners of the region the cut list declares for it.
    ///
    /// # Arguments
    /// * `index` - The index of the cut in `cuts`.
    ///
    /// # Returns
    /// An `Option` containing the hull points, or `None` for a cut that draws
    /// nothing or an index no cut occupies.
    pub fn outline(&self, index: usize) -> Option<Cow<'_, [(i32, i32)]>> {
        let cut = self.cuts.get(index)?;

        match self.hull.get(index) {
            Some(hull) => (!hull.is_empty()).then_some(Cow::Borrowed(hull.as_slice())),
            None => Some(Cow::Owned(vec![
                (cut.x, cut.y),
                (cut.x + cut.width, cut.y),
                (cut.x, cut.y + cut.height),
                (cut.x + cut.width, cut.y + cut.height),
            ])),
        }
    }

    /// Salvages a corrupted or truncated PNG stream into the largest decodable image.
    ///
    /// Chunk lengths and checksums are rewritten to match the bytes present, and
    /// the declared height is binary-searched downwards for the tallest prefix
    /// the decoder accepts. That prefix is composited onto a canvas of the
    /// original dimensions so sprite coordinates still resolve.
    ///
    /// Costs a full decode attempt per search step.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the damaged PNG stream.
    ///
    /// # Returns
    /// A `Result` containing the recovered image on success, or a `RigError` if
    /// the stream was too short to carry a header or no prefix of it decoded.
    pub fn repair(png: impl AsRef<[u8]>) -> Result<RgbaImage, RigError> {
        Self::repair_inner(png.as_ref())
    }

    fn repair_inner(png: &[u8]) -> Result<RgbaImage, RigError> {
        if png.len() < MINIMUM_PNG_LENGTH {
            return Err(RigError::TruncatedHeader);
        }

        fn calculate_crc32(chunk_type: &[u8], chunk_data: &[u8]) -> u32 {
            let mut crc_value = 0xFFFFFFFFu32;
            for &byte in chunk_type.iter().chain(chunk_data.iter()) {
                crc_value ^= byte as u32;
                for _ in 0..8 {
                    crc_value = if (crc_value & 1) != 0 {
                        (crc_value >> 1) ^ 0xEDB88320
                    } else {
                        crc_value >> 1
                    };
                }
            }
            crc_value ^ 0xFFFFFFFFu32
        }

        fn sanitize_png_chunks(bytes: &[u8]) -> Vec<u8> {
            let mut fixed = Vec::with_capacity(bytes.len() + 12);
            fixed.extend_from_slice(&bytes[0..8]);

            let mut read_offset = 8;
            let mut found_iend = false;

            while read_offset + 8 <= bytes.len() {
                let len_bytes = [bytes[read_offset], bytes[read_offset+1], bytes[read_offset+2], bytes[read_offset+3]];
                let chunk_len = u32::from_be_bytes(len_bytes) as usize;
                let chunk_type = &bytes[read_offset+4..read_offset+8];

                if read_offset + 8 + chunk_len + 4 > bytes.len() {
                    let avail = bytes.len() - (read_offset + 8);
                    fixed.extend_from_slice(&(avail as u32).to_be_bytes());
                    fixed.extend_from_slice(chunk_type);
                    fixed.extend_from_slice(&bytes[read_offset+8 .. read_offset+8+avail]);

                    let true_crc = calculate_crc32(chunk_type, &bytes[read_offset+8 .. read_offset+8+avail]);
                    fixed.extend_from_slice(&true_crc.to_be_bytes());
                    break;
                }

                let chunk_data = &bytes[read_offset+8 .. read_offset+8+chunk_len];
                let true_crc = calculate_crc32(chunk_type, chunk_data);

                fixed.extend_from_slice(&len_bytes);
                fixed.extend_from_slice(chunk_type);
                fixed.extend_from_slice(chunk_data);
                fixed.extend_from_slice(&true_crc.to_be_bytes());

                if chunk_type == b"IEND" {
                    found_iend = true;
                    break;
                }

                read_offset += 8 + chunk_len + 4;
            }

            if !found_iend {
                fixed.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]);
            }

            fixed
        }

        fn patch_png_height(sanitized: &[u8], new_height: u32) -> Vec<u8> {
            let mut patched = sanitized.to_vec();
            patched[20..24].copy_from_slice(&new_height.to_be_bytes());

            let crc = calculate_crc32(&patched[12..29], &[]);
            patched[29..33].copy_from_slice(&crc.to_be_bytes());

            patched
        }

        let sanitized = sanitize_png_chunks(png);

        if sanitized.len() < MINIMUM_PNG_LENGTH {
            return Err(RigError::TruncatedHeader);
        }

        let (Some(width_bytes), Some(height_bytes)) = (
            sanitized.get(16..20).and_then(|slice| <[u8; 4]>::try_from(slice).ok()),
            sanitized.get(20..24).and_then(|slice| <[u8; 4]>::try_from(slice).ok()),
        ) else {
            return Err(RigError::TruncatedHeader);
        };

        let orig_width = u32::from_be_bytes(width_bytes);
        let orig_height = u32::from_be_bytes(height_bytes).min(10000);

        let mut low = 1;
        let mut high = orig_height;
        let mut best_img = None;

        while low <= high {
            let mid = low + (high - low) / 2;
            let test_buffer = patch_png_height(&sanitized, mid);

            if let Ok(img) = image::load_from_memory(&test_buffer) {
                best_img = Some(img.to_rgba8());
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        let partial = best_img.ok_or(RigError::ImageDecodeFailed)?;

        let mut full_canvas = RgbaImage::new(orig_width, orig_height);
        for y in 0..partial.height() {
            for x in 0..partial.width() {
                full_canvas.put_pixel(x, y, *partial.get_pixel(x, y));
            }
        }

        Ok(full_canvas)
    }
}

/// Measures the visible extent of one cut and the hull enclosing its visible pixels.
///
/// The cut is clipped to the atlas first, so a region reaching past the edge is
/// measured over the part of it that exists. Only the topmost and bottommost
/// visible pixel of a column can touch the hull, so it is built from two
/// candidates per column rather than from every pixel.
///
/// The hull is held to [`HULL_CAP`] points, which encloses more than the pixels
/// themselves but never more than the rectangle already does.
fn measure(image: &RgbaImage, cut: &SpriteCut) -> Option<(Opaque, Vec<(i32, i32)>)> {
    let (atlas_width, atlas_height) = (image.width() as i64, image.height() as i64);

    let left = (cut.x as i64).clamp(0, atlas_width);
    let top = (cut.y as i64).clamp(0, atlas_height);
    let right = (cut.x as i64 + cut.width as i64).clamp(left, atlas_width);
    let bottom = (cut.y as i64 + cut.height as i64).clamp(top, atlas_height);

    let mut spans: Vec<Option<(i64, i64)>> = vec![None; (right - left) as usize];

    for y in top..bottom {
        for x in left..right {
            if image.get_pixel(x as u32, y as u32)[3] < ALPHA_FLOOR { continue; }

            let Some(span) = spans.get_mut((x - left) as usize) else { continue };

            *span = Some(span.map_or((y, y), |(first, _)| (first, y)));
        }
    }

    let (mut min_x, mut min_y) = (i64::MAX, i64::MAX);
    let (mut max_x, mut max_y) = (i64::MIN, i64::MIN);

    for (column, span) in spans.iter().enumerate() {
        let Some(&(first, last)) = span.as_ref() else { continue };

        min_x = min_x.min(left + column as i64);
        max_x = max_x.max(left + column as i64 + 1);
        min_y = min_y.min(first);
        max_y = max_y.max(last + 1);
    }

    if min_x >= max_x { return None; }

    let opaque = Opaque {
        x: min_x as i32,
        y: min_y as i32,
        width: (max_x - min_x) as i32,
        height: (max_y - min_y) as i32,
    };

    Some((opaque, bound(&spans, left, HULL_CAP)))
}

/// Wraps a span table in a hull of no more than `cap` points.
///
/// Columns are merged into ever wider groups and the hull rebuilt until it fits,
/// each group contributing the four corners of the rectangle enclosing its own
/// columns. A hull that already fits is the exact hull of the pixels.
fn bound(spans: &[Option<(i64, i64)>], left: i64, cap: usize) -> Vec<(i32, i32)> {
    let mut step = 1;

    let hull = loop {
        let hull = wrap(corners(spans, left, step));

        if hull.len() <= cap || step >= spans.len() { break hull; }

        step *= 2;
    };

    hull.into_iter().map(|(x, y)| (x as i32, y as i32)).collect()
}

/// Collects the hull candidates of one span table, merging every `step` columns into a group.
///
/// A group contributes the four corners of the rectangle enclosing its columns,
/// which contains every visible pixel those columns hold. A step of one leaves
/// each column its own group and yields the candidates of the exact hull.
fn corners(spans: &[Option<(i64, i64)>], left: i64, step: usize) -> Vec<(i64, i64)> {
    let mut points = Vec::with_capacity(spans.len().div_ceil(step.max(1)) * 4);

    for (group, columns) in spans.chunks(step.max(1)).enumerate() {
        let (mut near_x, mut near_y) = (i64::MAX, i64::MAX);
        let (mut far_x, mut far_y) = (i64::MIN, i64::MIN);

        for (offset, span) in columns.iter().enumerate() {
            let Some(&(first, last)) = span.as_ref() else { continue };

            let column = left + (group * step.max(1) + offset) as i64;

            near_x = near_x.min(column);
            near_y = near_y.min(first);
            far_x = far_x.max(column + 1);
            far_y = far_y.max(last + 1);
        }

        if near_x >= far_x { continue; }

        points.extend([(near_x, near_y), (far_x, near_y), (near_x, far_y), (far_x, far_y)]);
    }

    points
}

/// Wraps a set of points in the smallest convex polygon containing all of them.
///
/// The polygon comes back in counter-clockwise order with collinear points
/// dropped, so a rectangular set of candidates reduces to four points.
fn wrap(mut points: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    points.sort_unstable();
    points.dedup();

    if points.len() < 3 { return points; }

    let cross = |origin: (i64, i64), first: (i64, i64), second: (i64, i64)| {
        (first.0 - origin.0) * (second.1 - origin.1) - (first.1 - origin.1) * (second.0 - origin.0)
    };

    let turns_inward = |chain: &[(i64, i64)], point: (i64, i64)| {
        match (chain.len().checked_sub(2).and_then(|at| chain.get(at)), chain.last()) {
            (Some(&behind), Some(&last)) => cross(behind, last, point) <= 0,
            _ => false,
        }
    };

    let chain = |ordered: &mut dyn Iterator<Item = (i64, i64)>| {
        let mut half: Vec<(i64, i64)> = Vec::with_capacity(points.len());

        for point in ordered {
            while turns_inward(&half, point) { half.pop(); }

            half.push(point);
        }

        half.pop();
        half
    };

    let mut hull = chain(&mut points.iter().copied());
    hull.extend(chain(&mut points.iter().rev().copied()));

    hull
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas(rows: &[&str]) -> RgbaImage {
        let height = rows.len() as u32;
        let width = rows.first().map_or(0, |row| row.len()) as u32;

        RgbaImage::from_fn(width, height, |x, y| {
            let solid = rows[y as usize].as_bytes()[x as usize] == b'#';

            image::Rgba(if solid { [255, 255, 255, 255] } else { [0, 0, 0, 0] })
        })
    }

    fn png() -> Vec<u8> {
        let mut bytes = Vec::new();
        let image = RgbaImage::new(4, 4);

        image
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .map_or_else(|_| Vec::new(), |()| bytes)
    }

    fn list(header: &[&str], cuts: &[&str]) -> String {
        let mut lines: Vec<&str> = header.to_vec();
        lines.extend_from_slice(cuts);

        lines.join("\n")
    }

    #[test]
    fn three_header_lines_are_discarded_whatever_they_hold() {
        let sheet = SpriteSheet::parse(png(), list(&["[imgcut]", "1", "atlas.png", "1"], &["0,0,4,4,face"]))
            .expect("a cut list parses");

        assert_eq!(sheet.version, 1);
        assert_eq!(sheet.image_name, "atlas.png");
        assert_eq!(sheet.cuts.len(), 1);
        assert_eq!(sheet.cuts[0].name, "face");

        let shifted = SpriteSheet::parse(png(), list(&["", "[imgcut]", "1", "atlas.png", "1"], &["0,0,4,4"]));

        assert_eq!(shifted.err(), Some(RigError::NoSpriteCuts));
    }

    #[test]
    fn only_as_many_rows_are_read_as_the_count_declares() {
        let sheet = SpriteSheet::parse(png(), list(&["[imgcut]", "0", "atlas.png", "2"], &["0,0,1,1", "1,1,2,2", "2,2,3,3"]))
            .expect("a cut list parses");

        assert_eq!(sheet.cuts.len(), 2);

        let truncated = SpriteSheet::parse(png(), list(&["[imgcut]", "0", "atlas.png", "3"], &["0,0,1,1"]))
            .expect("a cut list parses");

        assert_eq!(truncated.cuts.len(), 3);
        assert_eq!(truncated.cuts[2], truncated.cuts[0]);

        assert_eq!(
            SpriteSheet::parse(png(), list(&["[imgcut]", "0", "atlas.png", "-1"], &[])).err(),
            Some(RigError::CountTooLarge),
        );
    }

    fn whole(image: &RgbaImage) -> SpriteCut {
        SpriteCut { x: 0, y: 0, width: image.width() as i32, height: image.height() as i32, name: String::new() }
    }

    fn cross(origin: (i32, i32), first: (i32, i32), second: (i32, i32)) -> i64 {
        let (first, second) = (
            ((first.0 - origin.0) as i64, (first.1 - origin.1) as i64),
            ((second.0 - origin.0) as i64, (second.1 - origin.1) as i64),
        );

        first.0 * second.1 - first.1 * second.0
    }

    fn encloses(hull: &[(i32, i32)], point: (i32, i32)) -> bool {
        hull.iter().enumerate().all(|(at, &corner)| {
            let next = hull[(at + 1) % hull.len()];

            cross(corner, next, point) >= 0
        })
    }

    #[test]
    fn cut_columns_map_one_field_each() {
        columns::assert_one_field_per_column(SpriteCut::COLUMNS);
    }

    #[test]
    fn a_solid_rectangle_hulls_to_its_four_corners() {
        let image = atlas(&["####", "####", "####"]);
        let (rect, hull) = measure(&image, &whole(&image)).expect("a solid atlas measures");

        assert_eq!(rect, Opaque { x: 0, y: 0, width: 4, height: 3 });
        assert_eq!(hull, vec![(0, 0), (4, 0), (4, 3), (0, 3)]);
    }

    #[test]
    fn a_diagonal_hulls_inside_the_rectangle_it_fills() {
        let image = atlas(&["#...", ".#..", "..#.", "...#"]);
        let (rect, hull) = measure(&image, &whole(&image)).expect("a diagonal atlas measures");

        assert_eq!(rect, Opaque { x: 0, y: 0, width: 4, height: 4 });
        assert!(hull.len() < 8);
        assert!(!encloses(&hull, (4, 0)));
        assert!(!encloses(&hull, (0, 4)));
    }

    #[test]
    fn a_transparent_cut_measures_nothing() {
        let image = atlas(&["....", "....", "...."]);

        assert!(measure(&image, &whole(&image)).is_none());
    }

    #[test]
    fn a_diamond_hulls_to_its_eight_sides() {
        let image = atlas(&["..##..", ".####.", "######", ".####.", "..##.."]);
        let (_, mut hull) = measure(&image, &whole(&image)).expect("a diamond measures");

        hull.sort_unstable();

        assert_eq!(hull, vec![(0, 2), (0, 3), (2, 0), (2, 5), (4, 0), (4, 5), (6, 2), (6, 3)]);
    }

    #[test]
    fn a_hull_past_the_cap_still_encloses_every_pixel() {
        let table: Vec<Option<(i64, i64)>> = (0..32).map(|column| Some((column * column / 4, 250))).collect();

        let exact = bound(&table, 0, 64);
        let capped = bound(&table, 0, 6);

        assert!(exact.len() > 6);
        assert!(capped.len() <= 6);

        for (column, span) in table.iter().enumerate() {
            let Some(&(first, last)) = span.as_ref() else { continue };
            let (near, far) = (column as i32, column as i32 + 1);

            for corner in [(near, first as i32), (far, first as i32), (near, last as i32 + 1), (far, last as i32 + 1)] {
                assert!(encloses(&capped, corner), "{corner:?} escaped the capped hull");
            }
        }
    }

    #[test]
    fn a_sheet_without_pixels_outlines_the_region_it_declares() {
        let sheet = SpriteSheet {
            cuts: vec![SpriteCut { x: 3, y: 5, width: 10, height: 20, name: String::new() }],
            ..Default::default()
        };

        assert_eq!(
            sheet.outline(0).as_deref(),
            Some([(3, 5), (13, 5), (3, 25), (13, 25)].as_slice()),
        );
        assert_eq!(sheet.outline(1), None);
    }

    #[test]
    fn a_cut_drawing_nothing_has_no_outline() {
        let sheet = SpriteSheet {
            cuts: vec![SpriteCut { x: 0, y: 0, width: 4, height: 4, name: String::new() }],
            opaque: vec![None],
            hull: vec![Vec::new()],
            ..Default::default()
        };

        assert_eq!(sheet.outline(0), None);
    }
}
