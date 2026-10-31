use codex::graphics::animate::FrameData;
use codex::graphics::rig::{Animation, BoundingBox, Rig};
use codex::graphics::tools::{joint, part};
use kokoro::domains::settings::AnimSettings;

use crate::platform::Shape;

use super::{Drag, Region};

const WORLD_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
const ORIGIN_MARK_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const WORLD_WIDTH: f32 = 1.5;
const ORIGIN_MARK_RADIUS: f32 = 5.0;
const PART_COLOR: [f32; 4] = [1.0, 0.35, 0.35, 0.32];
const PART_ORIGIN_COLOR: [f32; 4] = [0.3, 0.9, 1.0, 0.82];
const PART_WIDTH: f32 = 1.5;
const PART_ORIGIN_RADIUS: f32 = 2.4;

type Quad = [(f32, f32); 4];

const DIM: [f32; 4] = [0.0, 0.0, 0.0, 125.0 / 255.0];
const SELECTION_FILL: [f32; 4] = [1.0, 1.0, 0.0, 30.0 / 255.0];
const SELECTION_COLOR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const SELECTION_WIDTH: f32 = 2.0;
const REGION_WIDTH: f32 = 1.0;

pub(super) struct View {
    pub(super) size: (f32, f32),
    pub(super) pan: (f32, f32),
    pub(super) zoom: f32,
}

pub(super) struct Pose<'a> {
    pub(super) rig: &'a Rig,
    pub(super) anim: Option<&'a Animation>,
    pub(super) frame: i32,
    pub(super) offset: Option<usize>,
    pub(super) frames: &'a [FrameData],
}

impl View {
    fn to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (self.size.0 / 2.0 + (x + self.pan.0) * self.zoom, self.size.1 / 2.0 + (y + self.pan.1) * self.zoom)
    }
}

fn dim_outside(size: (f32, f32), min: (f32, f32), max: (f32, f32), shapes: &mut Vec<Shape>) {
    let left = min.0.clamp(0.0, size.0);
    let top = min.1.clamp(0.0, size.1);
    let right = max.0.clamp(0.0, size.0);
    let bottom = max.1.clamp(0.0, size.1);
    let band = |min: (f32, f32), max: (f32, f32)| Shape::Rect { min, max, fill: DIM, stroke: None };

    if top > 0.0 {
        shapes.push(band((0.0, 0.0), (size.0, top)));
    }

    if bottom < size.1 {
        shapes.push(band((0.0, bottom), (size.0, size.1)));
    }

    if left > 0.0 {
        shapes.push(band((0.0, top), (left, bottom)));
    }

    if right < size.0 {
        shapes.push(band((right, top), (size.0, bottom)));
    }
}

pub(super) fn camera(view: &View, selecting: bool, selection: Option<Drag>, region: Option<Region>) -> Vec<Shape> {
    let mut shapes = Vec::new();

    if selecting {
        match selection {
            Some((start, end)) => {
                let min = (start.0.min(end.0), start.1.min(end.1));
                let max = (start.0.max(end.0), start.1.max(end.1));

                dim_outside(view.size, min, max, &mut shapes);
                shapes.push(Shape::Rect { min, max, fill: SELECTION_FILL, stroke: Some((SELECTION_COLOR, SELECTION_WIDTH)) });
            }
            None => shapes.push(Shape::Rect { min: (0.0, 0.0), max: view.size, fill: DIM, stroke: None }),
        }

        return shapes;
    }

    if let Some(region) = region {
        let min = view.to_screen(region.x, region.y);
        let max = view.to_screen(region.x + region.w, region.y + region.h);

        dim_outside(view.size, min, max, &mut shapes);
        shapes.push(Shape::Rect { min, max, fill: [0.0; 4], stroke: Some((SELECTION_COLOR, REGION_WIDTH)) });
    }

    shapes
}

pub(super) fn shapes(pose: &Pose<'_>, reach: Option<BoundingBox>, view: &View, show: &AnimSettings) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let ground = view.to_screen(0.0, 0.0);

    if let Some(reach) = show.show_world.then_some(reach).flatten() {
        let left = view.to_screen(reach.min_x, 0.0).0;
        let right = view.to_screen(reach.max_x, 0.0).0;
        let top = view.to_screen(0.0, reach.min_y).1;

        shapes.push(Shape::Line { from: (left, ground.1), to: (right, ground.1), color: WORLD_COLOR, width: WORLD_WIDTH });

        if top < ground.1 {
            shapes.push(Shape::Line { from: ground, to: (ground.0, top), color: WORLD_COLOR, width: WORLD_WIDTH });
        }
    }

    if show.show_origin {
        shapes.push(Shape::Circle { center: ground, radius: ORIGIN_MARK_RADIUS, color: ORIGIN_MARK_COLOR });
    }

    if show.show_rig {
        outlines(pose, view, &mut shapes);
    }

    shapes
}

fn outlines(pose: &Pose<'_>, view: &View, shapes: &mut Vec<Shape>) {
    let joints = joint::resolve(pose.rig, pose.anim, pose.frame, pose.offset);
    let placed: Vec<(Option<usize>, Quad)> = match part::resolve(pose.rig, pose.anim, pose.frame, pose.offset) {
        Ok(mapped) => mapped.into_iter().map(|entry| (Some(entry.part), corners(&entry.frame, view))).collect(),
        Err(_) => pose.frames.iter().map(|frame| (None, corners(frame, view))).collect(),
    };

    for (index, quad) in placed {
        let origin = index
            .and_then(|part| joints.get(part))
            .and_then(|joint| joint.origin)
            .map_or_else(|| centroid(&quad), |(x, y)| view.to_screen(x, y));
        let [top_left, bottom_left, top_right, bottom_right] = quad;

        for (from, to) in [(top_left, top_right), (top_right, bottom_right), (bottom_right, bottom_left), (bottom_left, top_left)] {
            shapes.push(Shape::Line { from, to, color: PART_COLOR, width: PART_WIDTH });
        }

        shapes.push(Shape::Circle { center: origin, radius: PART_ORIGIN_RADIUS, color: PART_ORIGIN_COLOR });
    }
}

fn corners(frame: &FrameData, view: &View) -> Quad {
    std::array::from_fn(|at| view.to_screen(frame.vertices[at * 2], frame.vertices[at * 2 + 1]))
}

fn centroid(quad: &Quad) -> (f32, f32) {
    let sum = quad.iter().fold((0.0, 0.0), |held, at| (held.0 + at.0, held.1 + at.1));

    (sum.0 / 4.0, sum.1 / 4.0)
}
