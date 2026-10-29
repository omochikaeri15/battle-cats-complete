use std::error;
use std::fmt;
use std::mem;
use std::ptr;
use std::slice;

use crate::graphics::animate::{self, FrameData};
use crate::graphics::engine;
use crate::graphics::rig::{Animation, ModelPart, Rig};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PartError {
    CountMismatch {
        kept: usize,
        frames: usize,
    },
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

#[derive(Clone, Debug, PartialEq)]
pub struct PartFrame {
    pub part: usize,
    pub frame: FrameData,
}

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
    fn a_skipped_part_leaves_its_neighbors_on_their_own_indices() {
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
