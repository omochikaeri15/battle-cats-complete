use crate::graphics::animate::resolve_frame;
use crate::graphics::rig::{Animation, Rig, SpriteCut};

const INVISIBLE: f32 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl BoundingBox {
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    pub minimum_opacity: f32,
    pub minimum_glow_opacity: f32,
    pub maximum_scale: f32,
    pub scale_opacity_threshold: f32,
    pub maximum_vertical_stretch: f32,
    pub maximum_height_threshold: f32,
    pub minimum_y_bound: f32,
}

impl Tolerance {
    pub fn new(level: f32) -> Self {
        let level = level.clamp(0.0, 1.0);
        let inverse = 1.0 - level;

        Self {
            minimum_opacity: 0.01 + (0.24 * level),
            minimum_glow_opacity: 0.75 * level,
            maximum_scale: 3.0 + (inverse * 100.0),
            scale_opacity_threshold: 0.95 * level,
            maximum_vertical_stretch: 2.0 + (inverse * 50.0),
            maximum_height_threshold: 1000.0 + (inverse * 10000.0),
            minimum_y_bound: -1200.0 - (inverse * 10000.0),
        }
    }
}

pub fn calculate_animation_bounds(
    rig: &Rig,
    animations: &[&Animation],
    tolerance: Tolerance,
    to_frame: Option<i32>,
    offset: Option<usize>,
) -> Option<BoundingBox> {
    animations.iter().fold(None, |combined, animation| {
        let range = to_frame
            .map(|to| (0, animation.declared_frames().saturating_sub(1).min(to)));

        let measured = scan_bounds(rig, Some(animation), tolerance, range, offset);

        match (combined, measured) {
            (Some(combined), Some(measured)) => Some(combined.union(&measured)),
            (combined, measured) => combined.or(measured),
        }
    })
}

pub fn scan_bounds(
    rig: &Rig,
    animation: Option<&Animation>,
    tolerance: Tolerance,
    override_range: Option<(i32, i32)>,
    offset: Option<usize>,
) -> Option<BoundingBox> {
    let (start, end) = override_range.unwrap_or_else(|| {
        (0, animation.map_or(0, |animation| animation.declared_frames().saturating_sub(1)))
    });

    let mut bounds: Option<BoundingBox> = None;
    let mut mapped: Vec<(f32, f32)> = Vec::new();

    for frame in start..=end {
        for part in resolve_frame(rig, animation, frame, offset) {
            if part.opacity <= INVISIBLE || part.opacity < tolerance.minimum_opacity { continue; }
            if part.glow > 0 && part.opacity < tolerance.minimum_glow_opacity { continue; }

            let Some(cut) = rig.sheet.cuts.get(part.sprite_index) else { continue };
            let Some(outline) = rig.sheet.outline(part.sprite_index) else { continue };

            let corner = |at: usize| (part.vertices[2 * at], part.vertices[2 * at + 1]);
            let span = |from: usize, to: usize| {
                let ((from_x, from_y), (to_x, to_y)) = (corner(from), corner(to));
                (to_x - from_x).hypot(to_y - from_y)
            };

            let scale_x = if cut.width == 0 { 0.0 } else { span(0, 1) / cut.width as f32 };
            let scale_y = if cut.height == 0 { 0.0 } else { span(0, 2) / cut.height as f32 };

            if scale_x.max(scale_y) > tolerance.maximum_scale
                && (part.opacity < tolerance.scale_opacity_threshold || part.glow > 0) {
                continue;
            }

            let mut measured = BoundingBox { min_x: f32::MAX, min_y: f32::MAX, max_x: f32::MIN, max_y: f32::MIN };

            visible_corners(&part.vertices, cut, &outline, &mut mapped);

            for &(x, y) in mapped.iter() {
                measured.min_x = measured.min_x.min(x);
                measured.max_x = measured.max_x.max(x);
                measured.min_y = measured.min_y.min(y);
                measured.max_y = measured.max_y.max(y);
            }

            if measured.height() > tolerance.maximum_height_threshold
                && measured.height() > measured.width() * tolerance.maximum_vertical_stretch {
                continue;
            }

            if measured.max_y < tolerance.minimum_y_bound { continue; }

            bounds = Some(bounds.map_or(measured, |bounds| bounds.union(&measured)));
        }
    }

    bounds
}

fn visible_corners(vertices: &[f32; 8], cut: &SpriteCut, outline: &[(i32, i32)], mapped: &mut Vec<(f32, f32)>) {
    let at = |index: usize| (vertices[2 * index], vertices[2 * index + 1]);
    let (top_left, bottom_left, top_right, bottom_right) = (at(0), at(1), at(2), at(3));

    let point = |u: f32, v: f32| {
        let top = (
            top_left.0 + (top_right.0 - top_left.0) * u,
            top_left.1 + (top_right.1 - top_left.1) * u,
        );
        let bottom = (
            bottom_left.0 + (bottom_right.0 - bottom_left.0) * u,
            bottom_left.1 + (bottom_right.1 - bottom_left.1) * u,
        );

        (top.0 + (bottom.0 - top.0) * v, top.1 + (bottom.1 - top.1) * v)
    };

    mapped.clear();

    if cut.width <= 0 || cut.height <= 0 {
        mapped.extend([point(0.0, 0.0), point(0.0, 1.0), point(1.0, 0.0), point(1.0, 1.0)]);
        return;
    }

    let (width, height) = (cut.width as f64, cut.height as f64);

    mapped.extend(outline.iter().map(|&(x, y)| {
        let u = ((x as i64 - cut.x as i64) as f64 / width) as f32;
        let v = ((y as i64 - cut.y as i64) as f64 / height) as f32;

        point(u, v)
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(x: i32, y: i32, width: i32, height: i32) -> SpriteCut {
        SpriteCut { x, y, width, height, name: String::new() }
    }

    fn mapped(vertices: &[f32; 8], region: &SpriteCut, outline: &[(i32, i32)]) -> Vec<(f32, f32)> {
        let mut points = Vec::new();
        visible_corners(vertices, region, outline, &mut points);
        points
    }

    fn extent(vertices: &[f32; 8], region: &SpriteCut, outline: &[(i32, i32)]) -> BoundingBox {
        let start = BoundingBox { min_x: f32::MAX, min_y: f32::MAX, max_x: f32::MIN, max_y: f32::MIN };

        mapped(vertices, region, outline).iter().fold(start, |box_so_far, &(x, y)| BoundingBox {
            min_x: box_so_far.min_x.min(x),
            min_y: box_so_far.min_y.min(y),
            max_x: box_so_far.max_x.max(x),
            max_y: box_so_far.max_y.max(y),
        })
    }

    fn rectangle(visible: (i32, i32, i32, i32)) -> [(i32, i32); 4] {
        let (x, y, width, height) = visible;

        [(x, y), (x + width, y), (x, y + height), (x + width, y + height)]
    }

    #[test]
    fn a_cut_filled_to_its_edges_keeps_the_quad_it_was_given() {
        let vertices = [0.0, 0.0, 0.0, 40.0, 20.0, 0.0, 20.0, 40.0];
        let region = cut(5, 7, 20, 40);

        assert_eq!(
            mapped(&vertices, &region, &rectangle((5, 7, 20, 40))),
            [(0.0, 0.0), (20.0, 0.0), (0.0, 40.0), (20.0, 40.0)],
        );
    }

    #[test]
    fn a_padded_cut_shrinks_to_the_pixels_it_draws() {
        let vertices = [0.0, 0.0, 0.0, 40.0, 20.0, 0.0, 20.0, 40.0];
        let region = cut(0, 0, 20, 40);

        assert_eq!(
            mapped(&vertices, &region, &rectangle((5, 10, 10, 20))),
            [(5.0, 10.0), (15.0, 10.0), (5.0, 30.0), (15.0, 30.0)],
        );
    }

    #[test]
    fn a_rotated_quad_carries_the_visible_region_around_with_it() {
        let vertices = [0.0, 0.0, 20.0, 0.0, 0.0, -20.0, 20.0, -20.0];
        let region = cut(0, 0, 20, 20);

        assert_eq!(
            mapped(&vertices, &region, &rectangle((10, 0, 10, 20))),
            [(0.0, -10.0), (0.0, -20.0), (20.0, -10.0), (20.0, -20.0)],
        );
    }

    #[test]
    fn a_cut_declaring_no_area_is_left_alone() {
        let vertices = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let region = cut(0, 0, 0, 0);

        assert_eq!(
            mapped(&vertices, &region, &rectangle((0, 0, 0, 0))),
            [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0)],
        );
    }

    #[test]
    fn a_rectangular_sprite_measures_the_quad_it_fills_however_it_turns() {
        let region = cut(0, 0, 20, 20);
        let outline = rectangle((0, 0, 20, 20));

        let resting = [0.0, 0.0, 0.0, 20.0, 20.0, 0.0, 20.0, 20.0];
        let turned = [0.0, 0.0, 14.142136, 14.142136, 14.142136, -14.142136, 28.284271, 0.0];

        assert_eq!(mapped(&resting, &region, &outline).len(), 4);
        assert_eq!(
            extent(&resting, &region, &outline),
            BoundingBox { min_x: 0.0, min_y: 0.0, max_x: 20.0, max_y: 20.0 },
        );

        let corners = extent(&turned, &region, &outline);

        assert_eq!(mapped(&turned, &region, &outline).len(), 4);
        assert!((corners.min_x - 0.0).abs() < 0.001);
        assert!((corners.max_x - 28.284271).abs() < 0.001);
        assert!((corners.min_y + 14.142136).abs() < 0.001);
        assert!((corners.max_y - 14.142136).abs() < 0.001);
    }

    #[test]
    fn a_diagonal_sprite_turned_flat_measures_narrower_than_its_rectangle() {
        let region = cut(0, 0, 20, 20);
        let turned = [0.0, 0.0, 14.142136, 14.142136, 14.142136, -14.142136, 28.284271, 0.0];

        let whole = extent(&turned, &region, &rectangle((0, 0, 20, 20)));
        let diagonal = extent(&turned, &region, &[(0, 0), (2, 0), (20, 20), (18, 20)]);

        assert!(diagonal.height() < whole.height() / 4.0);
        assert!(diagonal.min_x >= whole.min_x && diagonal.max_x <= whole.max_x);
        assert!(diagonal.min_y >= whole.min_y && diagonal.max_y <= whole.max_y);
    }

    #[test]
    fn union_encloses_both_inputs() {
        let first = BoundingBox { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        let second = BoundingBox { min_x: 5.0, min_y: -5.0, max_x: 15.0, max_y: 5.0 };

        let combined = first.union(&second);

        assert_eq!(combined, BoundingBox { min_x: 0.0, min_y: -5.0, max_x: 15.0, max_y: 10.0 });
        assert_eq!(combined.width(), 15.0);
        assert_eq!(combined.height(), 15.0);
        assert_eq!(combined.center(), (7.5, 2.5));
    }

    #[test]
    fn tolerance_clamps_its_level() {
        let strict = Tolerance::new(1.0);

        assert_eq!(strict.minimum_opacity, 0.25);
        assert_eq!(strict.maximum_scale, 3.0);
        assert_eq!(Tolerance::new(5.0), strict);
    }
}
