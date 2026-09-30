//! The mapping from resolved geometry back to the model part that produced it.
//!
//! [`crate::graphics::animate::resolve_frame`] flattens a posed rig into
//! geometry and keeps no record of which part each quad came from, because the
//! draw order is a depth sort and the pass skips parts as it goes. This module
//! runs the same pass and reports the model index alongside each entry, so a
//! consumer holding a part index can find the geometry that part drew.

use std::error;
use std::fmt;
use std::mem;
use std::ptr;
use std::slice;

use crate::graphics::animate::{self, FrameData};
use crate::graphics::engine;
use crate::graphics::rig::{Animation, ModelPart, Rig};

/// Represents errors that can occur while mapping geometry back to model parts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PartError {
    /// The draw pass kept a different number of parts than it produced frames.
    CountMismatch {
        /// The number of parts the draw pass kept.
        kept: usize,
        /// The number of frames the draw pass produced.
        frames: usize,
    },
    /// A posed part did not borrow from the model it was posed against.
    UnmappedPart,
}

impl fmt::Display for PartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountMismatch { kept, frames } => {
                write!(f, "The draw pass kept {kept} parts but produced {frames} frames.")
            }
            Self::UnmappedPart => write!(f, "A posed part did not belong to the model it was posed against."),
        }
    }
}

impl error::Error for PartError {}

/// One entry of a resolved frame, paired with the model part that drew it.
#[derive(Clone, Debug, PartialEq)]
pub struct PartFrame {
    /// The index of the part within the model's own part list.
    pub part: usize,
    /// The geometry that part resolved to, identical to the entry [`animate::resolve_frame`] holds at the same position.
    pub frame: FrameData,
}

/// Resolves a rig at a single frame, reporting which model part drew each entry.
///
/// Recovering the part behind an entry means replaying the draw pass one part
/// at a time, so this costs more than [`animate::resolve_frame`] alone and
/// suits a diagnostic overlay rather than a draw loop.
///
/// # Arguments
/// * `rig` - The parsed rig supplying the part hierarchy and sprite atlas.
/// * `anim` - The animation to evaluate, or `None` to resolve the rig in its resting pose.
/// * `frame` - The frame to evaluate at. The engine has no notion of a fractional frame.
/// * `offset` - The index of the alignment row placing the rig, where zero is combat, or `None` to leave the rig at the engine's own origin.
///
/// # Returns
/// A `Result` containing a `Vec<PartFrame>` in the same order as
/// [`animate::resolve_frame`], or a `PartError` if the two passes disagreed on
/// which parts the engine draws.
pub fn resolve(
    rig: &Rig,
    anim: Option<&Animation>,
    frame: i32,
    offset: Option<usize>,
) -> Result<Vec<PartFrame>, PartError> {
    let (frames, posed) = animate::resolve_posed(rig, anim, frame, offset);

    let kept: Vec<&ModelPart> = posed.iter()
        .filter(|part| !engine::build(slice::from_ref(part), rig).is_empty())
        .map(|part| part.rest)
        .collect();

    if kept.len() != frames.len() {
        return Err(PartError::CountMismatch { kept: kept.len(), frames: frames.len() });
    }

    kept.into_iter().zip(frames)
        .map(|(rest, frame)| {
            index_of(rest, &rig.model.parts)
                .map(|part| PartFrame { part, frame })
                .ok_or(PartError::UnmappedPart)
        })
        .collect()
}

/// Locates a posed part's resting row within the model by its address.
///
/// Every posed part borrows a row of the same slice, so the distance from the
/// slice base names the row directly. The candidate is confirmed by address
/// before it is trusted, and a part borrowed from elsewhere falls back to a
/// scan.
pub(super) fn index_of(rest: &ModelPart, parts: &[ModelPart]) -> Option<usize> {
    let distance = (ptr::from_ref(rest) as usize).wrapping_sub(parts.as_ptr() as usize);
    let index = distance / mem::size_of::<ModelPart>();

    parts.get(index)
        .is_some_and(|candidate| ptr::eq(candidate, rest))
        .then_some(index)
        .or_else(|| parts.iter().position(|candidate| ptr::eq(candidate, rest)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::graphics::animate::resolve_frame;
    use crate::graphics::rig::{AnimModification, Keyframe, Model, SpriteCut, SpriteSheet};

    use super::*;

    fn part(parent: i32, id: i32, sprite: i32, z: i32, opacity: i32) -> ModelPart {
        ModelPart {
            parent,
            id,
            sprite,
            z,
            scale_x: 1000,
            scale_y: 1000,
            opacity,
            ..ModelPart::default()
        }
    }

    fn rig() -> Rig {
        let model = Model {
            parts: vec![
                part(-1, 0, 0, 5, 1000),
                part(0, -1, 0, 4, 1000),
                part(0, 0, 0, 3, 0),
                part(0, 0, 9, 2, 1000),
                part(0, 0, 1, 1, 1000),
                part(5, 0, 0, 0, 1000),
            ],
            ..Model::default()
        };

        let sheet = SpriteSheet {
            cuts: vec![
                SpriteCut { x: 0, y: 0, width: 10, height: 10, name: String::new() },
                SpriteCut { x: 10, y: 0, width: 20, height: 20, name: String::new() },
            ],
            ..SpriteSheet::default()
        };

        Rig { model, sheet }
    }

    fn animation() -> Animation {
        Animation {
            version: 1,
            modifications: vec![AnimModification {
                part: 4,
                kind: 2,
                keyframes: vec![
                    Keyframe { frame: 2, value: 1, ease: 0, ease_power: 0 },
                    Keyframe { frame: 4, value: 0, ease: 0, ease_power: 0 },
                ],
                ..AnimModification::default()
            }],
        }
    }

    #[test]
    fn every_frame_carries_the_part_that_drew_it() {
        let rig = rig();
        let mapped = resolve(&rig, None, 0, None).unwrap();

        let indices: Vec<usize> = mapped.iter().map(|entry| entry.part).collect();
        assert_eq!(indices, vec![4, 0]);

        let sprites: Vec<usize> = mapped.iter().map(|entry| entry.frame.sprite_index).collect();
        assert_eq!(sprites, vec![1, 0]);
    }

    #[test]
    fn the_count_agrees_with_the_geometry_pass_across_frames() {
        let rig = rig();
        let anim = animation();

        for frame in [0, 1, 2, 3, 4, 40] {
            let mapped = resolve(&rig, Some(&anim), frame, None).unwrap();
            let frames = resolve_frame(&rig, Some(&anim), frame, None);

            assert_eq!(mapped.len(), frames.len(), "frame {frame}");
        }
    }

    #[test]
    fn the_geometry_matches_the_geometry_pass_entry_for_entry() {
        let rig = rig();
        let anim = animation();

        for frame in [0, 3, 40] {
            for offset in [None, Some(0)] {
                let mapped = resolve(&rig, Some(&anim), frame, offset).unwrap();
                let frames = resolve_frame(&rig, Some(&anim), frame, offset);

                let geometry: Vec<FrameData> = mapped.into_iter().map(|entry| entry.frame).collect();
                assert_eq!(geometry, frames, "frame {frame}");
            }
        }
    }

    #[test]
    fn the_indices_are_valid_and_distinct() {
        let rig = rig();
        let anim = animation();

        for frame in [0, 2, 5] {
            let mapped = resolve(&rig, Some(&anim), frame, None).unwrap();
            let mut seen = HashSet::new();

            for entry in &mapped {
                assert!(entry.part < rig.model.parts.len());
                assert!(seen.insert(entry.part), "frame {frame} repeated part {}", entry.part);
            }
        }
    }

    #[test]
    fn a_skipped_part_leaves_its_neighbours_on_their_own_indices() {
        let rig = rig();
        let mapped = resolve(&rig, None, 0, None).unwrap();
        let indices: Vec<usize> = mapped.iter().map(|entry| entry.part).collect();

        assert!(!indices.contains(&1), "an undrawn part was emitted");
        assert!(!indices.contains(&2), "a fully transparent part was emitted");
        assert!(!indices.contains(&3), "a part outside the atlas was emitted");
        assert!(!indices.contains(&5), "a part outside the hierarchy was emitted");

        assert_eq!(indices, vec![4, 0]);
    }

    #[test]
    fn a_sprite_swap_keeps_the_part_it_swapped_on() {
        let rig = rig();
        let anim = animation();

        let swapped = resolve(&rig, Some(&anim), 40, None).unwrap();
        let entry = swapped.iter().find(|entry| entry.part == 4).unwrap();

        assert_eq!(entry.frame.sprite_index, 0);
        assert_eq!(swapped.iter().filter(|entry| entry.frame.sprite_index == 0).count(), 2);
    }

    #[test]
    fn every_row_of_the_model_finds_its_own_index() {
        let rig = rig();

        for (index, rest) in rig.model.parts.iter().enumerate() {
            assert_eq!(index_of(rest, &rig.model.parts), Some(index));
        }
    }

    #[test]
    fn a_part_borrowed_from_another_model_maps_to_nothing() {
        let (foreign, rig) = (rig(), rig());

        for rest in &foreign.model.parts {
            assert_eq!(index_of(rest, &rig.model.parts), None);
        }
    }

    #[test]
    fn a_rig_with_no_drawable_part_maps_to_nothing() {
        let mut rig = rig();
        rig.sheet.cuts.clear();

        assert_eq!(resolve(&rig, None, 0, None), Ok(Vec::new()));
    }
}
