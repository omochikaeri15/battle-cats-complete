use std::ops::Range;

use rayon::prelude::*;

const BAND_ROWS: usize = 16;
const LUMA: [f32; 3] = [0.299, 0.587, 0.114];
const TEXEL_SCALE: f32 = 1.0 / 255.0;
const OPAQUE_TEXEL: [f32; 4] = [1.0; 4];
const SUBTEXEL: u32 = 256;
const FILTERED_SCALE: f32 = 1.0 / (255.0 * SUBTEXEL as f32 * SUBTEXEL as f32);
const MULTIPLY: u8 = 2;
const ATTRIBUTES: usize = 10;
const UV: usize = 0;
const COLOR: usize = 2;
const TINT: usize = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub tint: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alpha {
    Blended,
    Carried,
}

#[derive(Clone, Copy)]
pub(crate) struct Texture<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

pub(crate) struct Batch<'a> {
    pub texture: Option<Texture<'a>>,
    pub blend: u8,
    pub alpha: Alpha,
    pub range: Range<usize>,
}

struct Plane {
    dx: f32,
    dy: f32,
    origin: f32,
}

impl Plane {
    fn at(&self, x: f32, y: f32) -> f32 {
        self.origin + self.dx * x + self.dy * y
    }
}

struct Edge {
    a: f32,
    b: f32,
    c: f32,
    inclusive: bool,
}

impl Edge {
    fn new(from: [f32; 2], to: [f32; 2]) -> Self {
        let a = from[1] - to[1];
        let b = to[0] - from[0];

        Self { a, b, c: from[0] * to[1] - from[1] * to[0], inclusive: a > 0.0 || (a == 0.0 && b < 0.0) }
    }

    fn at(&self, x: f32, y: f32) -> f32 {
        self.a * x + self.b * y + self.c
    }

    fn covers(&self, value: f32) -> bool {
        value > 0.0 || (value == 0.0 && self.inclusive)
    }

    fn clip(&self, y: f32, span: (f32, f32)) -> Option<(f32, f32)> {
        let rest = self.b * y + self.c;
        let (low, high) = span;

        match self.a.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Greater) => Some((low.max(-rest / self.a), high)),
            Some(std::cmp::Ordering::Less) => Some((low, high.min(-rest / self.a))),
            _ => self.covers(rest).then_some(span),
        }
    }
}

struct Triangle<'a> {
    edges: [Edge; 3],
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
    attributes: [Plane; ATTRIBUTES],
    tinted: bool,
    texture: Option<Texture<'a>>,
    blend: u8,
    alpha: Alpha,
}

fn plane(points: &[[f32; 2]; 3], values: [f32; 3], area: f32) -> Plane {
    let [p0, p1, p2] = *points;
    let dx = ((values[1] - values[0]) * (p2[1] - p0[1]) - (values[2] - values[0]) * (p1[1] - p0[1])) / area;
    let dy = ((values[2] - values[0]) * (p1[0] - p0[0]) - (values[1] - values[0]) * (p2[0] - p0[0])) / area;

    Plane { dx, dy, origin: values[0] - dx * p0[0] - dy * p0[1] }
}

fn attributes(vertex: &Vertex) -> [f32; ATTRIBUTES] {
    let [u, v] = vertex.uv;
    let [red, green, blue, alpha] = vertex.color;
    let [tint_red, tint_green, tint_blue, tint_alpha] = vertex.tint;

    [u, v, red, green, blue, alpha, tint_red, tint_green, tint_blue, tint_alpha]
}

fn floor(value: f32) -> f32 {
    let truncated = value as i64 as f32;

    if truncated > value { truncated - 1.0 } else { truncated }
}

impl<'a> Triangle<'a> {
    fn new(corners: &[Vertex], batch: &Batch<'a>, width: usize, height: usize) -> Option<Self> {
        let [v0, v1, v2] = corners else {
            return None;
        };
        let to_pixels = |vertex: &Vertex| [(vertex.position[0] + 1.0) * 0.5 * width as f32, (1.0 - vertex.position[1]) * 0.5 * height as f32];
        let p0 = to_pixels(v0);
        let (mut p1, mut p2) = (to_pixels(v1), to_pixels(v2));
        let a0 = v0;
        let (mut a1, mut a2) = (v1, v2);
        let mut area = (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p1[1] - p0[1]) * (p2[0] - p0[0]);

        if area == 0.0 || !area.is_finite() {
            return None;
        }

        if area < 0.0 {
            std::mem::swap(&mut p1, &mut p2);
            std::mem::swap(&mut a1, &mut a2);
            area = -area;
        }

        let points = [p0, p1, p2];
        let xs = [p0[0], p1[0], p2[0]];
        let ys = [p0[1], p1[1], p2[1]];
        let low_x = xs.iter().copied().fold(f32::INFINITY, f32::min).floor().max(0.0);
        let high_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max).ceil().min(width as f32);
        let low_y = ys.iter().copied().fold(f32::INFINITY, f32::min).floor().max(0.0);
        let high_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max).ceil().min(height as f32);

        if low_x >= high_x || low_y >= high_y {
            return None;
        }

        let tinted = [a0, a1, a2].iter().any(|vertex| vertex.tint[3] != 0.0);

        Some(Self {
            edges: [Edge::new(p1, p2), Edge::new(p2, p0), Edge::new(p0, p1)],
            left: low_x as usize,
            right: high_x as usize,
            top: low_y as usize,
            bottom: high_y as usize,
            attributes: {
                let corners = [attributes(a0), attributes(a1), attributes(a2)];

                std::array::from_fn(|index| plane(&points, [corners[0][index], corners[1][index], corners[2][index]], area))
            },
            tinted,
            texture: batch.texture,
            blend: batch.blend,
            alpha: batch.alpha,
        })
    }

    fn fill(&self, band: &mut [u8], first_row: usize, width: usize) {
        let rows = band.len() / (width * 4);
        let top = self.top.max(first_row);
        let bottom = self.bottom.min(first_row + rows);

        for row in top..bottom {
            let y = row as f32 + 0.5;
            let Some((low, high)) = self.edges.iter().try_fold((f32::NEG_INFINITY, f32::INFINITY), |span, edge| edge.clip(y, span)) else {
                continue;
            };

            if low > high {
                continue;
            }

            let start = ((low - 0.5).ceil() - 1.0).max(self.left as f32) as usize;
            let end = ((high - 0.5).floor() + 2.0).min(self.right as f32).max(0.0) as usize;
            let line = &mut band[(row - first_row) * width * 4..(row - first_row + 1) * width * 4];
            let first = start as f32 + 0.5;
            let mut values: [f32; ATTRIBUTES] = std::array::from_fn(|index| self.attributes[index].at(first, y));

            for column in start..end {
                let x = column as f32 + 0.5;
                let current = values;

                for (value, attribute) in values.iter_mut().zip(&self.attributes) {
                    *value += attribute.dx;
                }

                if !self.edges.iter().all(|edge| edge.covers(edge.at(x, y))) {
                    continue;
                }

                let texel = self.texture.map_or(OPAQUE_TEXEL, |texture| sample(texture, current[UV], current[UV + 1]));

                if self.blend != MULTIPLY && texel == [0.0; 4] {
                    continue;
                }

                let shaded = if self.tinted { wash(texel, [current[TINT], current[TINT + 1], current[TINT + 2], current[TINT + 3]]) } else { texel };
                let source: [f32; 4] = std::array::from_fn(|channel| shaded[channel] * current[COLOR + channel]);

                blend(&mut line[column * 4..column * 4 + 4], source, self.blend, self.alpha);
            }
        }
    }
}

fn texel(texture: Texture<'_>, x: i64, y: i64) -> [u32; 4] {
    let column = x.clamp(0, i64::from(texture.width) - 1) as usize;
    let row = y.clamp(0, i64::from(texture.height) - 1) as usize;
    let at = (row * texture.width as usize + column) * 4;

    texture.pixels.get(at..at + 4).map_or([0; 4], |pixel| std::array::from_fn(|channel| u32::from(pixel[channel])))
}

fn corners(texture: Texture<'_>, column: i64, row: i64) -> [[u32; 4]; 4] {
    let inside = column >= 0 && row >= 0 && column + 1 < i64::from(texture.width) && row + 1 < i64::from(texture.height);
    let stride = texture.width as usize * 4;
    let at = (row.max(0) as usize * texture.width as usize + column.max(0) as usize) * 4;
    let rows = inside.then(|| texture.pixels.get(at..at + 8).zip(texture.pixels.get(at + stride..at + stride + 8))).flatten();

    rows.map_or_else(
        || [texel(texture, column, row), texel(texture, column + 1, row), texel(texture, column, row + 1), texel(texture, column + 1, row + 1)],
        |(upper, lower)| {
            let read = |line: &[u8], offset: usize| std::array::from_fn(|channel| u32::from(line[offset + channel]));

            [read(upper, 0), read(upper, 4), read(lower, 0), read(lower, 4)]
        },
    )
}

fn sample(texture: Texture<'_>, u: f32, v: f32) -> [f32; 4] {
    if texture.width == 0 || texture.height == 0 {
        return [0.0; 4];
    }

    let x = u * texture.width as f32 - 0.5;
    let y = v * texture.height as f32 - 0.5;
    let (left, top) = (floor(x), floor(y));
    let across = (((x - left) * SUBTEXEL as f32 + 0.5) as u32).min(SUBTEXEL);
    let down = (((y - top) * SUBTEXEL as f32 + 0.5) as u32).min(SUBTEXEL);
    let [first, second, third, fourth] = corners(texture, left as i64, top as i64);

    std::array::from_fn(|channel| {
        let upper = first[channel] * (SUBTEXEL - across) + second[channel] * across;
        let lower = third[channel] * (SUBTEXEL - across) + fourth[channel] * across;

        (upper * (SUBTEXEL - down) + lower * down) as f32 * FILTERED_SCALE
    })
}

fn wash(texel: [f32; 4], tint: [f32; 4]) -> [f32; 4] {
    let lit = texel[0] * LUMA[0] + texel[1] * LUMA[1] + texel[2] * LUMA[2];
    let mix = |base: f32, target: f32| base * (1.0 - tint[3]) + target * lit * tint[3];

    [mix(texel[0], tint[0]), mix(texel[1], tint[1]), mix(texel[2], tint[2]), texel[3]]
}

fn blend(pixel: &mut [u8], source: [f32; 4], mode: u8, alpha: Alpha) {
    let target: [f32; 4] = std::array::from_fn(|channel| f32::from(pixel[channel]) * TEXEL_SCALE);
    let mixed = |channel: usize, coverage: f32| {
        let (src, dst) = (source[channel], target[channel]);

        match mode {
            1 => src + dst,
            2 => src * dst,
            3 => src + dst * (1.0 - coverage),
            _ => src + dst * (1.0 - source[3]),
        }
    };

    for channel in 0..3 {
        pixel[channel] = unorm(mixed(channel, source[channel]));
    }

    pixel[3] = match (alpha, mode) {
        (Alpha::Carried, 1..=3) => pixel[3],
        _ => unorm(mixed(3, source[3])),
    };
}

fn unorm(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

pub(crate) fn draw(pixels: &mut [u8], width: u32, height: u32, vertices: &[Vertex], batches: &[Batch<'_>]) {
    let (width, height) = (width as usize, height as usize);

    if width == 0 || height == 0 || pixels.len() < width * height * 4 {
        return;
    }

    let triangles: Vec<Triangle<'_>> = batches
        .iter()
        .flat_map(|batch| {
            vertices
                .get(batch.range.clone())
                .unwrap_or_default()
                .chunks_exact(3)
                .filter_map(move |corners| Triangle::new(corners, batch, width, height))
        })
        .collect();

    pixels[..width * height * 4].par_chunks_mut(width * 4 * BAND_ROWS).enumerate().for_each(|(band, rows)| {
        let first_row = band * BAND_ROWS;

        for triangle in triangles.iter().filter(|triangle| triangle.bottom > first_row && triangle.top < first_row + BAND_ROWS) {
            triangle.fill(rows, first_row, width);
        }
    });
}

pub(crate) fn straighten(pixels: &mut [u8]) {
    pixels.par_chunks_mut(4).for_each(|pixel| {
        let alpha = u32::from(pixel[3]);

        if alpha == 0 || alpha == 0xff {
            return;
        }

        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * 0xff + alpha / 2) / alpha).min(0xff) as u8;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{draw, Alpha, Batch, Texture, Vertex};

    fn quad(color: [f32; 4]) -> Vec<Vertex> {
        let corner = |x: f32, y: f32| Vertex { position: [x, y], uv: [(x + 1.0) / 2.0, (1.0 - y) / 2.0], color, tint: [0.0; 4] };

        vec![corner(-1.0, 1.0), corner(-1.0, -1.0), corner(1.0, -1.0), corner(-1.0, 1.0), corner(1.0, -1.0), corner(1.0, 1.0)]
    }

    #[test]
    fn a_quad_split_into_two_triangles_covers_every_pixel_once() {
        // The diagonal runs straight through pixel centers; drawing it twice would show up as a bright seam.
        let mut pixels = vec![0u8; 8 * 8 * 4];

        draw(&mut pixels, 8, 8, &quad([0.25; 4]), &[Batch { texture: None, blend: 1, alpha: Alpha::Blended, range: 0..6 }]);

        assert!(pixels.chunks_exact(4).all(|pixel| pixel == [64, 64, 64, 64]));
    }

    #[test]
    fn a_texture_drawn_at_its_own_size_comes_back_unchanged() {
        let texels: Vec<u8> = (0..16).map(|value| value * 16).collect();
        let mut pixels = vec![0u8; 2 * 2 * 4];
        let texture = Texture { width: 2, height: 2, pixels: &texels };

        draw(&mut pixels, 2, 2, &quad([1.0; 4]), &[Batch { texture: Some(texture), blend: 1, alpha: Alpha::Blended, range: 0..6 }]);

        assert_eq!(pixels, texels);
    }
}
