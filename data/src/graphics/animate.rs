//! The geometry a caller consumes, and the one call that produces it.
//!
//! This is the whole front of the graphics module. It holds no animation logic
//! of its own: `engine` poses the parts and flattens them into geometry, and
//! this places the result where the caller asked for it.

use super::engine;
use super::rig::{Animation, Model, Rig};

/// One part of a rig, resolved into renderer-ready geometry for a single frame.
///
/// The payload is deliberately backend-agnostic: it carries flat coordinate
/// arrays and a sprite index rather than any texture handle or draw call, so any
/// canvas implementation can consume it without the engine knowing about that
/// canvas.
///
/// The engine resolves a part all the way to four world-space corners and rounds
/// them to whole pixels, so `vertices` already holds that result and
/// `final_matrix` is the identity. It is kept so a consumer can still impose a
/// per-part transform of its own.
///
/// The four corners arrive in the order the engine submits them, which is the
/// order [`FrameData::INDICES`] draws two triangles from.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameData {
    /// The index of the sprite region in the rig's atlas that this part draws.
    pub sprite_index: usize,
    /// The identity, since the engine bakes each part's transform into `vertices`.
    pub final_matrix: [f32; 9],
    /// The part's corner positions as four consecutive x and y pairs.
    pub vertices: [f32; 8],
    /// The texture coordinates matching `vertices`, as four consecutive u and v pairs.
    pub uvs: [f32; 8],
    /// The part's resolved opacity, from fully transparent at zero to fully opaque at one.
    pub opacity: f32,
    /// The blending mode to draw the part with, where zero is ordinary alpha blending.
    pub glow: u8,
}

impl FrameData {
    /// The order the engine indexes a quad's four vertices in to draw it as two triangles.
    pub const INDICES: [u16; 6] = engine::INDICES;
}

impl Default for FrameData {
    fn default() -> Self {
        Self {
            sprite_index: 0,
            final_matrix: engine::IDENTITY,
            vertices: [0.0; 8],
            uvs: [0.0; 8],
            opacity: 0.0,
            glow: 0,
        }
    }
}

/// Resolves a rig at a single frame into renderer-ready geometry.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
/// * `offset` - The index of the alignment row placing the rig, where zero is combat, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A `Vec<FrameData>` containing one entry per drawn part, ordered so that
/// drawing them in sequence produces the correct depth layering.
pub fn resolve_frame(
    rig: &Rig,
    anim: Option<&Animation>,
    frame: i32,
    offset: Option<usize>,
) -> Vec<FrameData> {
    resolve_posed(rig, anim, frame, offset).0
}

/// Resolves a rig at a single frame, keeping the posed parts the geometry came from.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
/// * `offset` - The index of the alignment row placing the rig, where zero is combat, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A tuple holding the `Vec<FrameData>` [`resolve_frame`] returns and the
/// `Vec<engine::Part>` the pass produced it from, in the engine's draw order.
pub(super) fn resolve_posed<'a>(
    rig: &'a Rig,
    anim: Option<&Animation>,
    frame: i32,
    offset: Option<usize>,
) -> (Vec<FrameData>, Vec<engine::Part<'a>>) {
    let parts = engine::resolve(&rig.model, anim, frame, &rig.sheet);
    let mut frames = engine::build(&parts, rig);

    shift(&mut frames, lift(&parts, &rig.model, offset));

    (frames, parts)
}

/// Resolves the world point the rig's alignment row pins to the origin.
///
/// The engine resolves the row to a world point and draws the entity with that
/// point pinned to the position the entity occupies, so placing anything the
/// same pass placed means subtracting this point from it.
///
/// # Arguments
/// * `parts` - The posed parts, in any order.
/// * `model` - The model the parts were posed against.
/// * `offset` - The index of the alignment row placing the rig, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A tuple holding the x and y to subtract, which is zero when there is no row or the row resolves to nothing.
pub(super) fn lift(parts: &[engine::Part<'_>], model: &Model, offset: Option<usize>) -> (f32, f32) {
    offset
        .and_then(|row| engine::anchor(parts, model, row))
        .map_or((0.0, 0.0), |anchor| (anchor.x as f32, anchor.y as f32))
}

/// Translates every vertex by the negation of a lifted anchor.
fn shift(frames: &mut [FrameData], (x, y): (f32, f32)) {
    for frame in frames {
        for corner in frame.vertices.chunks_exact_mut(2) {
            corner[0] -= x;
            corner[1] -= y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::rig::{Alignment, AnimModification, Keyframe, ModelPart, SpriteCut, SpriteSheet};

    use super::*;

    fn box_part(parent: i32, x: i32, y: i32) -> ModelPart {
        ModelPart {
            parent,
            x,
            y,
            pivot_x: 32,
            pivot_y: 32,
            scale_x: 1000,
            scale_y: 1000,
            opacity: 1000,
            ..ModelPart::default()
        }
    }

    fn align(part: i32) -> Alignment {
        Alignment { part, x: 32, y: 64, ..Alignment::default() }
    }

    fn rig(parts: Vec<ModelPart>, alignment: Vec<Alignment>) -> Rig {
        let model = Model { version: 3, parts, alignment, ..Model::default() };

        let sheet = SpriteSheet {
            cuts: vec![SpriteCut { x: 0, y: 0, width: 64, height: 64, name: String::new() }],
            ..SpriteSheet::default()
        };

        Rig { model, sheet }
    }

    fn drive(part: i32, kind: i32, keyframes: Vec<Keyframe>) -> AnimModification {
        AnimModification { part, kind, keyframes, ..AnimModification::default() }
    }

    fn animation(modifications: Vec<AnimModification>) -> Animation {
        Animation { version: 1, modifications }
    }

    fn hold(value: i32) -> Vec<Keyframe> {
        vec![Keyframe { frame: 0, value, ease: 0, ease_power: 0 }]
    }

    fn centre(frames: &[FrameData], index: usize) -> (f32, f32) {
        let vertices = frames[index].vertices;
        let x = vertices.iter().step_by(2).sum::<f32>() / 4.0;
        let y = vertices.iter().skip(1).step_by(2).sum::<f32>() / 4.0;

        (x, y)
    }

    #[test]
    fn the_rig_orbits_an_alignment_row_whose_part_turns() {
        let rig = rig(vec![box_part(-1, 0, 0)], vec![align(0), align(0)]);
        let anim = animation(vec![
            drive(0, 11, vec![
                Keyframe { frame: 0, value: 0, ease: 0, ease_power: 0 },
                Keyframe { frame: 30, value: 3600, ease: 0, ease_power: 0 },
            ]),
            drive(0, 4, hold(30)),
            drive(0, 5, hold(-10)),
        ]);

        let resting = resolve_frame(&rig, Some(&anim), 0, Some(0));
        let turned = resolve_frame(&rig, Some(&anim), 15, Some(0));

        assert_eq!(centre(&resting, 0), (0.0, -32.0));
        assert_eq!(centre(&turned, 0), (0.0, 32.0));
    }

    #[test]
    fn the_row_is_measured_against_the_part_it_names() {
        let parts = vec![box_part(-1, 0, 0), box_part(0, 100, 0)];

        let root = resolve_frame(&rig(parts.clone(), vec![align(0)]), None, 0, Some(0));
        let child = resolve_frame(&rig(parts, vec![align(1)]), None, 0, Some(0));

        assert_eq!(centre(&root, 1), (100.0, -32.0));
        assert_eq!(centre(&child, 1), (0.0, -32.0));
    }

    #[test]
    fn the_row_subtracts_the_animated_pivot() {
        let rig = rig(vec![box_part(-1, 0, 0)], vec![align(0)]);
        let anim = animation(vec![drive(0, 6, hold(10))]);

        let frames = resolve_frame(&rig, Some(&anim), 0, Some(0));

        assert_eq!(centre(&frames, 0), (0.0, -32.0));
    }

    #[test]
    fn the_row_is_carried_by_the_animated_scale() {
        let rig = rig(vec![box_part(-1, 0, 0)], vec![align(0)]);
        let anim = animation(vec![drive(0, 10, hold(2000))]);

        let frames = resolve_frame(&rig, Some(&anim), 0, Some(0));

        assert_eq!(centre(&frames, 0), (0.0, -64.0));
    }
}
