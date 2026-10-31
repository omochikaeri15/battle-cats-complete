use std::collections::HashMap;

use tracing::debug;

use super::sink::Frame;
use super::text::label_key;

const BAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const PLAY_HEIGHT: f32 = 720.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub sheet: Option<Box<str>>,
    pub blend: u8,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub runs: Vec<Run>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Canvas {
    pub surfaced: bool,
    pub scale: f32,
    pub padded: bool,
}

impl Canvas {
    pub fn inset(self, bounds: Bounds) -> Option<(f32, f32)> {
        (self.surfaced && self.padded && self.scale < 1.0).then(|| ((bounds.width * (1.0 - self.scale) / 2.0).round(), (bounds.height * (1.0 - self.scale) / 2.0).round()))
    }

    pub fn unpad(self, x: f32, y: f32, bounds: Bounds) -> (f32, f32) {
        self.inset(bounds).map_or((x, y), |(left, top)| ((x - left) / self.scale, (y - top) / self.scale))
    }
}

struct Fit {
    scale: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn fit(bounds: Bounds, design_width: f32, aspect: f32) -> Fit {
    let wide = if design_width > 0.0 { bounds.width / design_width } else { 1.0 };

    if aspect <= 0.0 || design_width <= 0.0 {
        return Fit { scale: wide, x: 0.0, y: 0.0, width: bounds.width, height: bounds.height };
    }

    let scale = wide.min(bounds.height / (design_width * aspect));
    let width = design_width * scale;
    let height = design_width * aspect * scale;

    Fit { scale, x: (bounds.width - width) / 2.0, y: (bounds.height - height) / 2.0, width, height }
}

pub fn paint(frame: &Frame, sizes: &HashMap<Box<str>, (u32, u32)>, canvas: Canvas, bounds: Bounds) -> Scene {
    let Canvas { surfaced, .. } = canvas;
    let design_height = frame.design_height;
    let letterbox = frame.letterbox;
    let fit = fit(bounds, frame.design_width, frame.aspect);
    let scale = fit.scale;
    let mut vertices: Vec<Vertex> = Vec::with_capacity(frame.quads.len() * 6 + 6);
    let mut runs: Vec<Run> = Vec::new();
    let surface = |cx: f32, cy: f32| Vertex { position: [cx / bounds.width * 2.0 - 1.0, 1.0 - cy / bounds.height * 2.0], uv: [0.0, 0.0], color: BAR_COLOR };

    if surfaced {
        vertices.extend([
            surface(0.0, 0.0),
            surface(0.0, bounds.height),
            surface(bounds.width, bounds.height),
            surface(0.0, 0.0),
            surface(bounds.width, bounds.height),
            surface(bounds.width, 0.0),
        ]);
        runs.push(Run { sheet: None, blend: 0, start: 0, end: 6 });
    }

    let mut absent: Vec<&str> = Vec::new();
    let resolved: Vec<Option<(&str, (u32, u32))>> = frame.sheets.iter().map(|name| sizes.get(name).map(|size| (name.as_ref(), *size))).collect();

    for quad in &frame.quads {
        let label = quad.label.map(label_key);
        let sheet: Option<(&str, (u32, u32))> = match (quad.sheet, label.as_ref()) {
            (Some(index), _) => resolved.get(usize::from(index)).copied().flatten(),
            (None, Some(key)) => sizes.get(key).map(|size| (key.as_ref(), *size)),
            (None, None) => None,
        };

        if label.is_some() && sheet.is_none() {
            continue;
        }

        if let Some(name) = frame.sheet(quad)
            && sheet.is_none()
            && !absent.contains(&name)
        {
            absent.push(name);
        }

        let key = sheet.map(|(name, _)| name);
        let uv = sheet.as_ref().map_or([[0.0f32; 2]; 4], |(_, (width, height))| {
            let width = (*width).max(1) as f32;
            let height = (*height).max(1) as f32;
            let left = quad.source[0] / width;
            let top = quad.source[1] / height;
            let right = (quad.source[0] + quad.source[2]) / width;
            let bottom = (quad.source[1] + quad.source[3]) / height;

            [[left, top], [left, bottom], [right, bottom], [right, top]]
        });
        let clip = |point: [f32; 2]| [(point[0] * scale + fit.x) / bounds.width * 2.0 - 1.0, 1.0 - (point[1] * scale + fit.y) / bounds.height * 2.0];
        let corner = |slot: usize| Vertex { position: clip(quad.corners[slot]), uv: uv[slot], color: quad.colors[slot] };
        let start = vertices.len() as u32;

        vertices.extend([corner(0), corner(1), corner(2), corner(0), corner(2), corner(3)]);

        let end = vertices.len() as u32;

        match runs.last_mut() {
            Some(last) if last.sheet.as_deref() == key && last.blend == quad.blend => last.end = end,
            _ => runs.push(Run { sheet: key.map(Box::from), blend: quad.blend, start, end }),
        }
    }

    let pad = if surfaced { letterbox.max(0.0) * scale } else { 0.0 };
    let floor = fit.y + (design_height - letterbox.max(0.0)).min(letterbox.max(0.0) + PLAY_HEIGHT) * scale;
    let mut bars = vec![[0.0, fit.y, bounds.width, pad], [0.0, floor, bounds.width, if pad > 0.0 { bounds.height - floor } else { 0.0 }]];

    if frame.aspect > 0.0 {
        bars.extend([
            [0.0, 0.0, bounds.width, fit.y],
            [0.0, fit.y + fit.height, bounds.width, bounds.height - fit.y - fit.height],
            [0.0, fit.y, fit.x, fit.height],
            [fit.x + fit.width, fit.y, bounds.width - fit.x - fit.width, fit.height],
        ]);
    }
    let start = vertices.len() as u32;

    for [x, y, width, height] in bars.into_iter().filter(|bar| bar[2] > 0.0 && bar[3] > 0.0) {
        vertices.extend([surface(x, y), surface(x, y + height), surface(x + width, y + height), surface(x, y), surface(x + width, y + height), surface(x + width, y)]);
    }

    let end = vertices.len() as u32;

    if end > start {
        runs.push(Run { sheet: None, blend: 0, start, end });
    }

    if !absent.is_empty() {
        debug!("emu: {} sheets drawn but never decoded: {absent:?}", absent.len());
    }

    Scene { vertices, runs }
}
