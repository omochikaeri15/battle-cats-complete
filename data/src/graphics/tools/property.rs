//! The published map from an [`AnimModification::kind`](crate::graphics::rig::AnimModification::kind) to the pose field it drives.
//!
//! `engine::animate`'s `match modification.kind` decides which property of a
//! part's pose each kind number touches, and until now that decision lived
//! nowhere a caller could read it back. This module restates it as data, in
//! kind order, so a consumer labeling or authoring a modification does not
//! have to mirror the engine's arms by hand.

/// How the engine's `animate` step folds a modification's value into the part's pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    /// The value replaces the rest value, stored as a difference from it.
    Offset,
    /// The value is the pose outright.
    Absolute,
}

/// One entry of the engine's `kind` map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Property {
    /// The [`AnimModification::kind`](crate::graphics::rig::AnimModification::kind) this entry describes.
    pub kind: i32,
    /// The field it drives, named as the engine's own pose struct names it, or
    /// `"scale"` for kind 8, which names no single field of its own.
    pub field: &'static str,
    /// How `animate` folds it into the pose.
    pub blend: Blend,
    /// Whether the kind drives more than one field, as kind 8 drives both scales.
    pub paired: bool,
}

/// The engine's `kind` map, in order.
pub const PROPERTIES: &[Property] = &[
    Property { kind: 0, field: "parent", blend: Blend::Offset, paired: false },
    Property { kind: 1, field: "id", blend: Blend::Offset, paired: false },
    Property { kind: 2, field: "sprite", blend: Blend::Offset, paired: false },
    Property { kind: 3, field: "depth", blend: Blend::Offset, paired: false },
    Property { kind: 4, field: "x", blend: Blend::Absolute, paired: false },
    Property { kind: 5, field: "y", blend: Blend::Absolute, paired: false },
    Property { kind: 6, field: "pivot_x", blend: Blend::Absolute, paired: false },
    Property { kind: 7, field: "pivot_y", blend: Blend::Absolute, paired: false },
    Property { kind: 8, field: "scale", blend: Blend::Absolute, paired: true },
    Property { kind: 9, field: "scale_x", blend: Blend::Absolute, paired: false },
    Property { kind: 10, field: "scale_y", blend: Blend::Absolute, paired: false },
    Property { kind: 11, field: "angle", blend: Blend::Absolute, paired: false },
    Property { kind: 12, field: "opacity", blend: Blend::Absolute, paired: false },
    Property { kind: 13, field: "flip_x", blend: Blend::Absolute, paired: false },
    Property { kind: 14, field: "flip_y", blend: Blend::Absolute, paired: false },
];

/// The entry for a kind, or `None` for one the engine ignores.
///
/// # Arguments
/// * `kind` - The [`AnimModification::kind`](crate::graphics::rig::AnimModification::kind) to look up.
///
/// # Returns
/// An `Option` containing the matching `Property`, or `None` if the engine's
/// `match` leaves that kind to its catch-all arm.
pub fn property(kind: i32) -> Option<&'static Property> {
    PROPERTIES.iter().find(|entry| entry.kind == kind)
}

#[cfg(test)]
mod tests {
    use crate::graphics::animate::resolve_frame;
    use crate::graphics::rig::{AnimModification, Animation, Keyframe, Model, ModelPart, Rig, SpriteCut, SpriteSheet};

    use super::*;

    fn root() -> ModelPart {
        ModelPart { parent: -1, id: 0, sprite: 0, z: 0, scale_x: 2000, scale_y: 1000, opacity: 1000, ..ModelPart::default() }
    }

    fn marker() -> ModelPart {
        ModelPart { parent: 0, id: 2, sprite: 2, z: 5, scale_x: 1000, scale_y: 1000, opacity: 1000, ..ModelPart::default() }
    }

    fn subject() -> ModelPart {
        ModelPart { parent: 0, id: 1, sprite: 1, z: 1, scale_x: 1000, scale_y: 1000, opacity: 1000, ..ModelPart::default() }
    }

    fn sheet() -> SpriteSheet {
        SpriteSheet {
            cuts: vec![
                SpriteCut { x: 0, y: 0, width: 10, height: 10, name: String::new() },
                SpriteCut { x: 10, y: 0, width: 20, height: 20, name: String::new() },
                SpriteCut { x: 30, y: 0, width: 5, height: 5, name: String::new() },
            ],
            ..SpriteSheet::default()
        }
    }

    fn rig(subject: ModelPart) -> Rig {
        let model = Model { parts: vec![root(), subject, marker()], ..Model::default() };
        Rig { model, sheet: sheet() }
    }

    fn animation(kind: i32, value: i32) -> Animation {
        Animation {
            version: 1,
            modifications: vec![AnimModification {
                part: 1,
                kind,
                keyframes: vec![Keyframe { frame: 0, value, ease: 0, ease_power: 0 }],
                ..AnimModification::default()
            }],
        }
    }

    /// Mutates a rest part the way `field` claims the engine's pose does, and
    /// returns the value an animation on that field must carry to match it.
    fn scenario(field: &str) -> (i32, ModelPart) {
        let mut rest = subject();

        let value = match field {
            "parent" => { rest.parent = -1; -1 }
            "id" => { rest.id = -1; -1 }
            "sprite" => { rest.sprite = 2; 2 }
            "depth" => { rest.z = 10; 10 }
            "x" => { rest.x = 40; 40 }
            "y" => { rest.y = 45; 45 }
            "pivot_x" => { rest.pivot_x = 3; 3 }
            "pivot_y" => { rest.pivot_y = 4; 4 }
            "scale" => { rest.scale_x = 1600; rest.scale_y = 1600; 1600 }
            "scale_x" => { rest.scale_x = 2000; 2000 }
            "scale_y" => { rest.scale_y = 1500; 1500 }
            "angle" => { rest.angle = 900; 900 }
            "opacity" => { rest.opacity = 500; 500 }
            "flip_x" => { rest.scale_x = -rest.scale_x; 1 }
            "flip_y" => { rest.scale_y = -rest.scale_y; 1 }
            other => panic!("scenario has no fixture for field {other}"),
        };

        (value, rest)
    }

    #[test]
    fn properties_cover_every_kind_in_order() {
        let kinds: Vec<i32> = PROPERTIES.iter().map(|entry| entry.kind).collect();
        assert_eq!(kinds, (0..=14).collect::<Vec<i32>>());
    }

    #[test]
    fn zero_to_three_are_offset_and_four_to_fourteen_are_absolute() {
        for property in PROPERTIES {
            let expected = if property.kind <= 3 { Blend::Offset } else { Blend::Absolute };
            assert_eq!(property.blend, expected, "kind {}", property.kind);
        }
    }

    #[test]
    fn only_kind_eight_is_paired() {
        for property in PROPERTIES {
            assert_eq!(property.paired, property.kind == 8, "kind {}", property.kind);
        }
    }

    #[test]
    fn property_looks_up_the_matching_entry() {
        for entry in PROPERTIES {
            assert_eq!(property(entry.kind), Some(entry));
        }

        assert_eq!(property(-1), None);
        assert_eq!(property(15), None);
    }

    #[test]
    fn every_kind_reproduces_directly_setting_its_rest_field() {
        let default = resolve_frame(&rig(subject()), None, 0, None);

        for property in PROPERTIES {
            let (value, expected_rest) = scenario(property.field);

            let animated = resolve_frame(&rig(subject()), Some(&animation(property.kind, value)), 0, None);
            let baseline = resolve_frame(&rig(expected_rest), None, 0, None);

            assert_eq!(animated, baseline, "kind {} field {} disagreed with the engine", property.kind, property.field);
            assert_ne!(animated, default, "kind {} field {} moved nothing", property.kind, property.field);
        }
    }

    #[test]
    fn only_the_published_kinds_move_anything() {
        let untouched = resolve_frame(&rig(subject()), None, 0, None);

        for kind in [-1, 15, 100] {
            let moved = resolve_frame(&rig(subject()), Some(&animation(kind, 77)), 0, None);
            assert_eq!(moved, untouched, "kind {kind}");
        }
    }
}
