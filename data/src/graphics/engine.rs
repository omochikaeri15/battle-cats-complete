//! Logic replicating the games animation engine.
//!
//! One pass over a rig does four things, in order: clear each part's animated
//! state, evaluate the animation into it, place every part in the world parent
//! before child, and sort the parts by depth. Everything but the rotation is
//! integer arithmetic, and every division truncates toward zero.
//!
//! Nothing here is public. A final draw pass flattens the placed parts into the
//! geometry a caller consumes, mirroring the engine's own draw call.

use std::ptr;

use super::animate::FrameData;
use super::rig::{AnimModification, Animation, Keyframe, Model, ModelPart, Rig, SpriteSheet};

/// Interpolation applied between a keyframe and the one that follows it.
const EASE_LINEAR: i32 = 0;
const EASE_HOLD: i32 = 1;
const EASE_EXPONENTIAL: i32 = 2;
const EASE_POLYNOMIAL: i32 = 3;

/// The fixed point the polynomial easing accumulates its terms in.
const POLYNOMIAL_SHIFT: u32 = 12;

/// The parent index that marks a part as a root of the hierarchy.
const NO_PARENT: i32 = -1;

/// The identifier that marks a part the engine never draws.
const NOT_DRAWN: i32 = -1;

/// The highest blending mode defined.
const GLOW_MODES: i32 = 3;

/// The row-major three by three identity, which every part carries.
pub(super) const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// The order the engine indexes a quad's four vertices in, drawing two triangles.
pub(super) const INDICES: [u16; 6] = [0, 1, 2, 3, 2, 1];

/// A whole-pixel position, which is the only precision the engine keeps corners at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Point {
    pub(super) x: i32,
    pub(super) y: i32,
}

/// The two by three affine the engine composes down the part hierarchy.
///
/// A part inherits its parent's transform, shifts by its own parent-scaled
/// offset, and then turns by its own angle, so composing it onto a corner walks
/// the whole ancestry outwards. The engine stores it as six consecutive floats
/// and applies it to whole-pixel points, which is why every corner comes back
/// truncated toward zero rather than kept at sub-pixel precision.
#[derive(Clone, Copy, Debug)]
pub(super) struct Transform {
    x_axis: (f32, f32),
    y_axis: (f32, f32),
    origin: (f32, f32),
}

impl Default for Transform {
    fn default() -> Self {
        Self { x_axis: (1.0, 0.0), y_axis: (0.0, 1.0), origin: (0.0, 0.0) }
    }
}

impl Transform {
    fn at(x: i32, y: i32) -> Self {
        Self { origin: (x as f32, y as f32), ..Self::default() }
    }

    fn translate(&mut self, x: i32, y: i32) {
        let (x, y) = (x as f32, y as f32);

        self.origin.0 += self.x_axis.0 * x + self.y_axis.0 * y;
        self.origin.1 += self.x_axis.1 * x + self.y_axis.1 * y;
    }

    /// Turns the transform, with the angle in degrees because the engine
    /// normalizes a full turn to three hundred and sixty before it gets here.
    ///
    /// The engine widens to double for the conversion and the trigonometry and
    /// narrows only the result, which matters because the caller truncates the
    /// transformed corners to whole pixels.
    fn rotate(&mut self, degrees: f32) {
        let radians = degrees as f64 * std::f64::consts::PI / 180.0;
        let (sine, cosine) = (radians.sin() as f32, radians.cos() as f32);
        let (x_axis, y_axis) = (self.x_axis, self.y_axis);

        self.x_axis = (
            x_axis.0 * cosine + y_axis.0 * sine,
            x_axis.1 * cosine + y_axis.1 * sine,
        );
        self.y_axis = (
            y_axis.0 * cosine - x_axis.0 * sine,
            y_axis.1 * cosine - x_axis.1 * sine,
        );
    }

    /// Applies the transform, truncating toward zero as the engine's own cast to
    /// a signed integer does.
    pub(super) fn apply(&self, point: Point) -> Point {
        let (x, y) = (point.x as f32, point.y as f32);

        Point {
            x: (self.x_axis.0 * x + self.y_axis.0 * y + self.origin.0) as i32,
            y: (self.x_axis.1 * x + self.y_axis.1 * y + self.origin.1) as i32,
        }
    }
}

/// The animated state of one part for a single frame.
///
/// The engine keeps this apart from the model's resting values and clears it
/// before every pass, so a modification writes an offset here rather than
/// touching the part it drives. Every field is cleared to the value that leaves
/// the resting pose alone, which is zero for what the engine adds and the
/// model's own unit for what it scales. A modification assigns its field
/// outright, so two modifications driving one property leave only the later one
/// standing.
#[derive(Clone, Copy, Debug)]
struct Pose {
    parent: i32,
    id: i32,
    sprite: i32,
    depth: i32,
    x: i32,
    y: i32,
    pivot_x: i32,
    pivot_y: i32,
    scale_x: i32,
    scale_y: i32,
    angle: i32,
    opacity: i32,
    flip_x: u8,
    flip_y: u8,
}

impl Pose {
    fn cleared(model: &Model) -> Self {
        Self {
            parent: 0,
            id: 0,
            sprite: 0,
            depth: 0,
            x: 0,
            y: 0,
            pivot_x: 0,
            pivot_y: 0,
            scale_x: model.scale_unit,
            scale_y: model.scale_unit,
            angle: 0,
            opacity: model.opacity_unit,
            flip_x: 0,
            flip_y: 0,
        }
    }
}

/// One part's placement in the world, written by the deployment pass.
///
/// The scale and opacity stay in the model's own units rather than becoming
/// ratios, because the engine keeps dividing by those units as it walks down the
/// hierarchy.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct World {
    scale_x: i32,
    scale_y: i32,
    opacity: i32,
    flip_x: bool,
    flip_y: bool,
    corners: [Point; 4],
    pub(super) transform: Transform,
}

/// One model part being posed, holding the block the engine keeps per part.
#[derive(Clone, Copy, Debug)]
pub(super) struct Part<'a> {
    pub(super) rest: &'a ModelPart,
    pose: Pose,
    pub(super) world: World,
}

impl Part<'_> {
    pub(super) fn parent(&self) -> i32 {
        self.rest.parent.wrapping_add(self.pose.parent)
    }

    fn id(&self) -> i32 {
        self.rest.id.wrapping_add(self.pose.id)
    }

    fn sprite(&self) -> i32 {
        self.rest.sprite.wrapping_add(self.pose.sprite)
    }

    fn depth(&self) -> i32 {
        self.rest.z.wrapping_add(self.pose.depth)
    }

    /// Whether the engine draws this part at all.
    fn drawn(&self) -> bool {
        self.id() != NOT_DRAWN
    }

    /// The atlas region this part draws, which the caller must still bounds check.
    fn region(&self) -> i32 {
        self.sprite()
    }

    /// The part's four world-space corners, clockwise from the top left.
    fn corners(&self) -> [Point; 4] {
        self.world.corners
    }

    /// The part's resolved opacity as the eight bit value the engine draws with.
    ///
    /// The engine quantizes here, on an integer divide, and skips the part
    /// entirely when the result is zero.
    fn alpha(&self, opacity_unit: i32) -> i32 {
        over_unit(self.world.opacity.wrapping_mul(u8::MAX as i32) as i64, opacity_unit) as i32
    }

    /// The blending mode the model declares for this part.
    fn glow(&self) -> i32 {
        self.rest.glow
    }
}

/// Divides by one of the model's units, truncating toward zero as the engine does.
///
/// A zero divisor leaves the value untouched, which is the closest defined
/// outcome to the division fault the engine would take on a malformed model.
/// The result stays wide because the engine narrows only where it stores.
fn over_unit(value: i64, unit: i32) -> i64 {
    if unit == 0 { return value; }

    value / unit as i64
}

/// Poses a model at a frame and places every part in the world.
///
/// The returned parts are in the engine's draw order: sorted on resolved depth
/// with a stable sort, so parts sharing a depth keep the order the model
/// declares them in.
pub(super) fn resolve<'a>(
    model: &'a Model,
    anim: Option<&Animation>,
    frame: i32,
    sheet: &SpriteSheet,
) -> Vec<Part<'a>> {
    let mut parts: Vec<Part<'a>> = model.parts.iter()
        .map(|rest| Part { rest, pose: Pose::cleared(model), world: World::default() })
        .collect();

    if let Some(animation) = anim {
        animate(model, animation, frame, &mut parts);
    }

    for index in deployment_order(&parts) {
        deploy(&mut parts, index, model, sheet);
    }

    parts.sort_by_key(Part::depth);

    parts
}

/// Applies an animation to a cleared pose at a given frame.
///
/// A modification assigns the field it drives rather than adding to it, so when
/// two modifications drive one property of one part only the later one survives.
/// The four properties the engine stores as an offset from the model instead
/// record the difference, which resolves back to the value evaluated here.
fn animate(model: &Model, animation: &Animation, frame: i32, parts: &mut [Part]) {
    for modification in &animation.modifications {
        let Some(value) = evaluate(modification, frame) else { continue };

        let Ok(index) = usize::try_from(modification.part) else { continue };
        let (Some(part), Some(rest)) = (parts.get_mut(index), model.parts.get(index)) else { continue };

        match modification.kind {
            0 => part.pose.parent = value.wrapping_sub(rest.parent),
            1 => part.pose.id = value.wrapping_sub(rest.id),
            2 => part.pose.sprite = value.wrapping_sub(rest.sprite),
            3 => part.pose.depth = value.wrapping_sub(rest.z),
            4 => part.pose.x = value,
            5 => part.pose.y = value,
            6 => part.pose.pivot_x = value,
            7 => part.pose.pivot_y = value,
            8 => {
                part.pose.scale_y = value;
                part.pose.scale_x = value;
            }
            9 => part.pose.scale_x = value,
            10 => part.pose.scale_y = value,
            11 => part.pose.angle = value,
            12 => part.pose.opacity = value,
            13 => part.pose.flip_x = value as u8,
            14 => part.pose.flip_y = value as u8,
            _ => {}
        }
    }
}

/// Resolves the value a modification holds at a frame.
///
/// Returns `None` before the modification's first keyframe, where the engine
/// leaves the property alone entirely.
pub(super) fn evaluate(modification: &AnimModification, frame: i32) -> Option<i32> {
    let keyframes = &modification.keyframes;
    let first = keyframes.first()?;
    let last = keyframes.last()?;

    if first.frame > frame { return None; }

    let span = last.frame.wrapping_sub(first.frame);

    if span == 0 { return Some(first.value); }

    let local = local_frame(modification, first.frame, last.frame, span, frame);

    if local == last.frame { return Some(last.value); }

    let mut index = 0;
    let mut bracketed = false;

    while index + 1 < keyframes.len() {
        if local >= keyframes[index].frame && local < keyframes[index + 1].frame {
            bracketed = true;
            break;
        }

        index += 1;
    }

    if !bracketed { return Some(0); }

    let start = keyframes[index];
    let end = keyframes[index + 1];
    let progress = local.wrapping_sub(start.frame);
    let width = end.frame.wrapping_sub(start.frame);

    Some(match start.ease {
        EASE_LINEAR => start.value.wrapping_add(
            progress.wrapping_mul(end.value.wrapping_sub(start.value)).wrapping_div(width),
        ),
        EASE_HOLD => start.value,
        EASE_EXPONENTIAL => exponential(start, end, progress, width),
        EASE_POLYNOMIAL => polynomial(keyframes, index, local),
        _ => 0,
    })
}

/// Folds a frame back into a modification's own keyframe range.
///
/// A modification replaying forever wraps over its span every time. One with a
/// replay count wraps only while replays remain and then rests on its final
/// keyframe, which is also where any other replay count lands immediately.
pub(super) fn local_frame(modification: &AnimModification, first: i32, last: i32, span: i32, frame: i32) -> i32 {
    if last > frame { return frame; }

    let elapsed = frame.wrapping_sub(first) as i64;
    let span = span as i64;
    let wrapped = || first.wrapping_add((elapsed % span) as i32);

    match modification.loop_count {
        -1 => wrapped(),
        count if count > 0 && elapsed / span < count as i64 => wrapped(),
        _ => last,
    }
}

/// Eases along a power curve, mirroring the engine's mix of double precision and
/// truncation back to an integer.
fn exponential(start: Keyframe, end: Keyframe, progress: i32, width: i32) -> i32 {
    let ratio = progress as f64 / width as f64;
    let power = start.ease_power as f64;

    let eased = if start.ease_power < 0 {
        (1.0 - (1.0 - ratio).powf(-power)).sqrt()
    } else {
        1.0 - (1.0 - ratio.powf(power)).sqrt()
    };

    (end.value.wrapping_sub(start.value) as f64 * eased + start.value as f64) as i32
}

/// Interpolates across the whole run of consecutive polynomial keyframes the
/// current one belongs to.
///
/// Terms widen before they are shifted into fixed point and accumulate there, so
/// only the single shift back down at the end loses anything.
fn polynomial(keyframes: &[Keyframe], index: usize, local: i32) -> i32 {
    let mut low = index;
    while low > 0 && keyframes[low - 1].ease == EASE_POLYNOMIAL {
        low -= 1;
    }

    let mut high = index + 1;
    while high + 1 < keyframes.len() && keyframes[high].ease == EASE_POLYNOMIAL {
        high += 1;
    }

    let mut total: i64 = 0;

    for outer in low..=high {
        let mut term = (keyframes[outer].value as i64) << POLYNOMIAL_SHIFT;

        for inner in low..=high {
            if outer == inner { continue; }

            let divisor = keyframes[outer].frame.wrapping_sub(keyframes[inner].frame) as i64;
            if divisor == 0 { continue; }

            term = term
                .wrapping_mul(local.wrapping_sub(keyframes[inner].frame) as i64)
                .wrapping_div(divisor);
        }

        total = total.wrapping_add(term);
    }

    (total / (1 << POLYNOMIAL_SHIFT)) as i32
}

/// Orders the parts so that every part follows its parent.
///
/// The engine sweeps the whole model once per generation, collecting the parts
/// whose resolved parent was collected in the previous sweep and starting from
/// the parts that name no parent at all. A part caught in a parent cycle is
/// never collected, and so is never placed in the world.
pub(super) fn deployment_order(parts: &[Part]) -> Vec<usize> {
    let mut order = Vec::with_capacity(parts.len());
    let mut frontier = vec![NO_PARENT];

    while !frontier.is_empty() {
        let mut collected = Vec::new();

        for (index, part) in parts.iter().enumerate() {
            if !frontier.contains(&part.parent()) { continue; }

            order.push(index);
            collected.push(index as i32);
        }

        frontier = collected;
    }

    order
}

/// Places one part in the world, given that its parent already has been.
///
/// A part whose sprite region does not exist leaves both its corners and its
/// transform untouched, which is what the engine does by returning before it
/// reaches either.
fn deploy(parts: &mut [Part], index: usize, model: &Model, sheet: &SpriteSheet) {
    let parent = parts[index].parent();

    // The engine roots a part on its parent being the sentinel, not on the index
    // failing to resolve. Only the traversal reaches here, and it never collects
    // a part whose parent is neither the sentinel nor a real index.
    let anchor = (parent != NO_PARENT)
        .then(|| usize::try_from(parent).ok().and_then(|at| parts.get(at)))
        .flatten()
        .map(|part| part.world);

    let (scale_unit, opacity_unit) = (model.scale_unit, model.opacity_unit);
    let part = &mut parts[index];
    let (rest, pose) = (&part.rest, &part.pose);

    match anchor {
        None => {
            let root = |resting: i32, animated: i32, unit: i32| {
                over_unit(resting.wrapping_mul(animated) as i64, unit) as i32
            };

            part.world.scale_x = root(rest.scale_x, pose.scale_x, scale_unit);
            part.world.scale_y = root(pose.scale_y, rest.scale_y, scale_unit);
            part.world.opacity = root(pose.opacity, rest.opacity, opacity_unit);
            part.world.flip_x = pose.flip_x != 0;
            part.world.flip_y = pose.flip_y != 0;
        }
        Some(anchor) => {
            let inherited = |outer: i32, animated: i32, resting: i32, unit: i32| {
                let product = (outer as i64) * (animated as i64) * (resting as i64);
                over_unit(over_unit(product, unit), unit) as i32
            };

            part.world.scale_x = inherited(anchor.scale_x, pose.scale_x, rest.scale_x, scale_unit);
            part.world.scale_y = inherited(anchor.scale_y, pose.scale_y, rest.scale_y, scale_unit);
            part.world.opacity = inherited(anchor.opacity, pose.opacity, rest.opacity, opacity_unit);
            part.world.flip_x = anchor.flip_x ^ (pose.flip_x != 0);
            part.world.flip_y = anchor.flip_y ^ (pose.flip_y != 0);
        }
    }

    if pose.flip_x != 0 { part.world.scale_x = part.world.scale_x.wrapping_neg(); }
    if pose.flip_y != 0 { part.world.scale_y = part.world.scale_y.wrapping_neg(); }

    if part.id() != NOT_DRAWN {
        let Ok(sprite) = usize::try_from(part.sprite()) else { return };
        let Some(cut) = sheet.cuts.get(sprite) else { return };

        let (scale_x, scale_y) = (part.world.scale_x, part.world.scale_y);

        let pivot_x = pose.pivot_x.wrapping_add(rest.pivot_x).wrapping_mul(scale_x).wrapping_neg();
        let pivot_y = pose.pivot_y.wrapping_add(rest.pivot_y).wrapping_mul(scale_y).wrapping_neg();

        let left = over_unit(pivot_x as i64, scale_unit) as i32;
        let top = over_unit(pivot_y as i64, scale_unit) as i32;
        let right = left.wrapping_add(over_unit(cut.width.wrapping_mul(scale_x) as i64, scale_unit) as i32);
        let bottom = top.wrapping_add(over_unit(cut.height.wrapping_mul(scale_y) as i64, scale_unit) as i32);

        part.world.corners = [
            Point { x: left, y: top },
            Point { x: left, y: bottom },
            Point { x: right, y: bottom },
            Point { x: right, y: top },
        ];
    }

    part.world.transform = match anchor {
        None => Transform::at(pose.x, pose.y),
        Some(anchor) => {
            let mut transform = anchor.transform;

            transform.translate(
                over_unit(pose.x.wrapping_add(rest.x).wrapping_mul(anchor.scale_x) as i64, scale_unit) as i32,
                over_unit(pose.y.wrapping_add(rest.y).wrapping_mul(anchor.scale_y) as i64, scale_unit) as i32,
            );

            transform
        }
    };

    let turn = (pose.angle.wrapping_add(rest.angle)) as f32 * 360.0 / model.angle_unit as f32;
    let turn = if part.world.flip_x == part.world.flip_y { turn } else { -turn };

    part.world.transform.rotate(turn);

    for corner in &mut part.world.corners {
        *corner = part.world.transform.apply(*corner);
    }
}

/// Resolves the world point an alignment row places the rig by.
///
/// The row names the part it is measured against and an offset from that part's
/// pivot. The pivot the engine subtracts is the animated one, the scale it then
/// applies is the part's world scale composed down the hierarchy, and the pair
/// is carried into the world by the same matrix the part's own corners went
/// through, so a row whose part turns makes the whole rig swing about the point.
///
/// The engine reads the row and the part it names unchecked, both of which
/// `tools::crash` already reports, so a row that names neither resolves to
/// nothing here rather than faulting.
pub(super) fn anchor(parts: &[Part<'_>], model: &Model, row: usize) -> Option<Point> {
    let align = model.alignment.get(row)?;
    let rest = usize::try_from(align.part).ok().and_then(|index| model.parts.get(index))?;
    let part = parts.iter().find(|placed| ptr::eq(placed.rest, rest))?;

    let local = |offset: i32, resting: i32, animated: i32, scale: i32| {
        let pivot = resting.wrapping_add(animated);

        over_unit(offset.wrapping_sub(pivot) as i64 * scale as i64, model.scale_unit) as i32
    };

    Some(part.world.transform.apply(Point {
        x: local(align.x, part.rest.pivot_x, part.pose.pivot_x, part.world.scale_x),
        y: local(align.y, part.rest.pivot_y, part.pose.pivot_y, part.world.scale_y),
    }))
}

/// Flattens the placed parts into one quad each, in the order they arrive.
///
/// A part the engine never draws, one naming an atlas region that does not
/// exist, and one whose opacity quantizes away to nothing are all dropped, which
/// is what the engine's own draw call does with each of them.
///
/// A blending mode the engine's table does not define leaves the mode the last
/// part set still in force, since the mode lives on the draw context rather than
/// on the part.
///
/// The four corners are emitted in the order the engine submits them, pairing
/// the near edge of the cut rect with the left corners and the far edge with the
/// right, so a consumer drawing them through [`FrameData::INDICES`] reproduces
/// the engine's own two triangles and its winding.
pub(super) fn build(parts: &[Part<'_>], rig: &Rig) -> Vec<FrameData> {
    let sheet = &rig.sheet;
    let (atlas_width, atlas_height) = sheet.image_data.as_ref()
        .map_or((1.0, 1.0), |image| (image.width() as f32, image.height() as f32));

    let opacity_unit = rig.model.opacity_unit;
    let mut frames = Vec::with_capacity(parts.len());
    let mut glow = 0;

    for part in parts {
        if !part.drawn() { continue; }

        if part.glow() <= GLOW_MODES {
            glow = part.glow().max(0) as u8;
        }

        let alpha = part.alpha(opacity_unit);
        if alpha == 0 { continue; }

        let Ok(sprite) = usize::try_from(part.region()) else { continue };
        let Some(cut) = sheet.cuts.get(sprite) else { continue };

        let [top_left, bottom_left, bottom_right, top_right] = part.corners();

        let near_u = cut.x as f32 / atlas_width;
        let far_u = cut.x.wrapping_add(cut.width) as f32 / atlas_width;
        let near_v = cut.y as f32 / atlas_height;
        let far_v = cut.y.wrapping_add(cut.height) as f32 / atlas_height;

        frames.push(FrameData {
            sprite_index: sprite,
            final_matrix: IDENTITY,
            vertices: [
                top_left.x as f32, top_left.y as f32,
                bottom_left.x as f32, bottom_left.y as f32,
                top_right.x as f32, top_right.y as f32,
                bottom_right.x as f32, bottom_right.y as f32,
            ],
            uvs: [
                near_u, near_v,
                near_u, far_v,
                far_u, near_v,
                far_u, far_v,
            ],
            opacity: alpha as f32 / u8::MAX as f32,
            glow,
        });
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyframe(frame: i32, value: i32, ease: i32) -> Keyframe {
        Keyframe { frame, value, ease, ease_power: 0 }
    }

    fn modification(keyframes: Vec<Keyframe>) -> AnimModification {
        AnimModification { loop_count: 1, keyframes, ..AnimModification::default() }
    }

    #[test]
    fn evaluate_returns_none_before_the_first_keyframe() {
        let modification = modification(vec![
            keyframe(10, 5, EASE_LINEAR),
            keyframe(20, 15, EASE_LINEAR),
        ]);

        assert_eq!(evaluate(&modification, 5), None);
    }

    #[test]
    fn evaluate_returns_the_last_value_at_the_last_frame() {
        let modification = modification(vec![
            keyframe(0, 5, EASE_LINEAR),
            keyframe(20, 15, EASE_LINEAR),
        ]);

        assert_eq!(evaluate(&modification, 20), Some(15));
    }

    #[test]
    fn evaluate_finds_the_first_matching_bracket_even_when_a_later_keyframe_repeats_the_frame() {
        // A real `.maanim` file (unit 360's true form, second animation, part
        // 27's angle curve) declares frames out of order: ..., 21, 17, 26, ...
        // The engine's own forward scan brackets on the first pair it finds,
        // so a later keyframe naming a frame the scan already passed is
        // reachable only through whichever pair the scan hits first.
        let modification = modification(vec![
            keyframe(0, 0, EASE_LINEAR),
            keyframe(14, 14, EASE_LINEAR),
            keyframe(15, 15, EASE_LINEAR),
            keyframe(21, 21, EASE_LINEAR),
            keyframe(17, 17, EASE_LINEAR),
            keyframe(26, 26, EASE_LINEAR),
        ]);

        assert_eq!(evaluate(&modification, 17), Some(17));
        assert_eq!(evaluate(&modification, 20), Some(20));
    }
}
