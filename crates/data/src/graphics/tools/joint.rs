//! Where each part of a posed rig turns about, and which part it hangs from.
//!
//! [`crate::graphics::animate::resolve_frame`] bakes every part into four
//! corners, which loses the point a part rotates about and the parent the
//! animation resolved it to. This module runs the same pass without the draw
//! step and reads both back out for every part in the model's own order.

use crate::graphics::animate;
use crate::graphics::engine;
use crate::graphics::rig::{Animation, Rig};

use super::part;

/// Where one part turns about and which part it hangs from, once the animation is applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joint {
    /// The index of the part within the model's own part list.
    pub part: usize,
    /// The part it hangs from after the animation's parent curve, or `None` where the engine anchors it to nothing.
    pub parent: Option<usize>,
    /// The world point the part's pivot lands on, placed by the same alignment row as its geometry, or `None` where deployment never reaches the part.
    pub origin: Option<(f32, f32)>,
}

/// Resolves every part's joint at a single frame.
///
/// A part's pivot is the origin of its own local space, so the point it turns
/// about is that origin carried through its world transform. A part the engine
/// reaches but cannot find a sprite region for keeps the identity transform,
/// and its children really are anchored to it.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
/// * `offset` - The index of the alignment row placing the rig, where zero is combat, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A `Vec<Joint>` in the model's own part order, one entry per part, drawn or not.
pub fn resolve(rig: &Rig, anim: Option<&Animation>, frame: i32, offset: Option<usize>) -> Vec<Joint> {
    let model = &rig.model;
    let mut parts = engine::resolve(model, anim, frame, &rig.sheet);

    parts.sort_by_key(|placed| part::index_of(placed.rest, &model.parts));

    let (x, y) = animate::lift(&parts, model, offset);
    let mut reached = vec![false; parts.len()];

    for index in engine::deployment_order(&parts) {
        if let Some(flag) = reached.get_mut(index) {
            *flag = true;
        }
    }

    parts.iter().zip(reached).enumerate()
        .map(|(index, (placed, reached))| Joint {
            part: index,
            parent: usize::try_from(placed.parent()).ok().filter(|&at| at < parts.len()),
            origin: reached.then(|| {
                let pivot = placed.world.transform.apply(engine::Point::default());

                (pivot.x as f32 - x, pivot.y as f32 - y)
            }),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::graphics::rig::{Alignment, AnimModification, Keyframe, Model, ModelPart, SpriteCut, SpriteSheet};
    use crate::graphics::tools::part::PartFrame;

    use super::*;

    const REPARENT_AT: i32 = 5;
    const ANIMATED_PIVOT: i32 = 10;

    fn part(parent: i32, sprite: i32, z: i32, at: (i32, i32), pivot: (i32, i32), scale: i32) -> ModelPart {
        ModelPart {
            parent,
            sprite,
            z,
            x: at.0,
            y: at.1,
            pivot_x: pivot.0,
            pivot_y: pivot.1,
            scale_x: scale,
            scale_y: scale,
            opacity: 1000,
            ..ModelPart::default()
        }
    }

    fn rig() -> Rig {
        let mut hidden = part(1, 0, 0, (5, 5), (8, 8), 1000);
        hidden.id = -1;

        let model = Model {
            version: 3,
            parts: vec![
                part(-1, 0, 3, (0, 0), (20, 20), 1000),
                part(0, 0, 1, (30, -5), (32, 32), 2000),
                part(0, 1, 2, (-12, 7), (10, 5), 1000),
                hidden,
                part(3, 0, 5, (3, 3), (16, 48), 1000),
                part(9, 0, -1, (1, 1), (0, 0), 1000),
                part(5, 0, 4, (2, 2), (0, 0), 1000),
                part(1, 9, -2, (0, 0), (0, 0), 1000),
                part(7, 0, 6, (4, 6), (0, 0), 1000),
            ],
            alignment: vec![Alignment { part: 1, x: 32, y: 64, ..Alignment::default() }],
            ..Model::default()
        };

        let sheet = SpriteSheet {
            cuts: vec![
                SpriteCut { x: 0, y: 0, width: 64, height: 64, name: String::new() },
                SpriteCut { x: 64, y: 0, width: 40, height: 20, name: String::new() },
            ],
            ..SpriteSheet::default()
        };

        Rig { model, sheet }
    }

    fn hold(frame: i32, value: i32) -> Vec<Keyframe> {
        vec![Keyframe { frame, value, ease: 1, ease_power: 0 }]
    }

    fn animation() -> Animation {
        Animation {
            version: 1,
            modifications: vec![
                AnimModification { part: 0, kind: 6, keyframes: hold(0, ANIMATED_PIVOT), ..AnimModification::default() },
                AnimModification { part: 2, kind: 0, keyframes: hold(REPARENT_AT, 1), ..AnimModification::default() },
            ],
        }
    }

    fn bilinear_pivot(rig: &Rig, entry: &PartFrame, animated: bool) -> (f32, f32) {
        let rest = &rig.model.parts[entry.part];
        let cut = &rig.sheet.cuts[entry.frame.sprite_index];
        let extra = if animated && entry.part == 0 { ANIMATED_PIVOT } else { 0 };

        let u = (rest.pivot_x + extra) as f32 / cut.width as f32;
        let v = rest.pivot_y as f32 / cut.height as f32;
        let [left, top, _, bottom, right, _, _, _] = entry.frame.vertices;

        (left + u * (right - left), top + v * (bottom - top))
    }

    #[test]
    fn the_parts_come_back_in_model_order_despite_mixed_depths() {
        let rig = rig();
        let joints = resolve(&rig, None, 0, None);

        assert_eq!(joints.len(), rig.model.parts.len());

        for (index, (joint, rest)) in joints.iter().zip(&rig.model.parts).enumerate() {
            assert_eq!(joint.part, index);
            assert_eq!(joint.parent, usize::try_from(rest.parent).ok().filter(|&at| at < joints.len()));
        }
    }

    #[test]
    fn the_parent_curve_reparents_only_once_it_is_keyed() {
        let rig = rig();
        let anim = animation();

        assert_eq!(resolve(&rig, Some(&anim), REPARENT_AT - 1, None)[2].parent, Some(0));
        assert_eq!(resolve(&rig, Some(&anim), REPARENT_AT, None)[2].parent, Some(1));
    }

    #[test]
    fn the_origin_sits_on_the_pivot_of_every_drawn_quad() {
        let rig = rig();
        let anim = animation();

        for (anim, animated) in [(None, false), (Some(&anim), true)] {
            for frame in [0, REPARENT_AT + 3] {
                for offset in [None, Some(0)] {
                    let joints = resolve(&rig, anim, frame, offset);
                    let drawn = part::resolve(&rig, anim, frame, offset).unwrap();

                    assert!(!drawn.is_empty());

                    for entry in &drawn {
                        let expected = bilinear_pivot(&rig, entry, animated);
                        assert_eq!(joints[entry.part].origin, Some(expected), "part {} frame {frame}", entry.part);
                    }
                }
            }
        }
    }

    #[test]
    fn the_animated_pivot_moves_the_quad_and_not_the_origin() {
        let rig = rig();
        let anim = animation();

        let resting = part::resolve(&rig, None, 0, None).unwrap();
        let animated = part::resolve(&rig, Some(&anim), 0, None).unwrap();
        let root = |frames: &[PartFrame]| frames.iter().find(|entry| entry.part == 0).unwrap().frame.vertices[0];

        assert_eq!(root(&animated), root(&resting) - ANIMATED_PIVOT as f32);
        assert_eq!(resolve(&rig, None, 0, None)[0].origin, resolve(&rig, Some(&anim), 0, None)[0].origin);
    }

    #[test]
    fn the_alignment_row_shifts_every_origin_with_its_vertices() {
        let rig = rig();
        let unplaced = resolve(&rig, None, 0, None);
        let placed = resolve(&rig, None, 0, Some(0));
        let before = part::resolve(&rig, None, 0, None).unwrap();
        let after = part::resolve(&rig, None, 0, Some(0)).unwrap();

        let (moved_x, moved_y) = (after[0].frame.vertices[0] - before[0].frame.vertices[0], after[0].frame.vertices[1] - before[0].frame.vertices[1]);
        assert_ne!((moved_x, moved_y), (0.0, 0.0));

        for (from, to) in unplaced.iter().zip(&placed) {
            let (Some(start), Some(end)) = (from.origin, to.origin) else {
                assert_eq!(from.origin, to.origin);
                continue;
            };

            assert_eq!((end.0 - start.0, end.1 - start.1), (moved_x, moved_y), "part {}", from.part);
        }
    }

    #[test]
    fn a_dangling_parent_leaves_the_part_and_its_children_unplaced() {
        let rig = rig();
        let joints = resolve(&rig, None, 0, None);

        assert_eq!(joints[5].parent, None);
        assert_eq!(joints[5].origin, None);
        assert_eq!(joints[6].parent, Some(5));
        assert_eq!(joints[6].origin, None);
    }

    #[test]
    fn an_undrawn_part_still_carries_its_children() {
        let rig = rig();
        let joints = resolve(&rig, None, 0, None);

        assert_eq!(joints[1].origin, Some((30.0, -5.0)));
        assert_eq!(joints[3].origin, Some((40.0, 5.0)));
        assert_eq!(joints[4].origin, Some((46.0, 11.0)));
    }

    #[test]
    fn a_missing_cut_leaves_the_identity_its_children_anchor_to() {
        let rig = rig();
        let joints = resolve(&rig, None, 0, None);

        assert_eq!(joints[7].origin, Some((0.0, 0.0)));
        assert_eq!(joints[8].origin, Some((8.0, 12.0)));
    }
}
