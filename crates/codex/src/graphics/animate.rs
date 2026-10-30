use super::engine;
use super::rig::{Animation, Model, Rig};

#[derive(Clone, Debug, PartialEq)]
pub struct FrameData {
    pub sprite_index: usize,
    pub final_matrix: [f32; 9],
    pub vertices: [f32; 8],
    pub uvs: [f32; 8],
    pub opacity: f32,
    pub glow: u8,
}

impl FrameData {
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

pub fn resolve_frame(
    rig: &Rig,
    anim: Option<&Animation>,
    frame: i32,
    offset: Option<usize>,
) -> Vec<FrameData> {
    resolve_posed(rig, anim, frame, offset).0
}

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

pub(super) fn lift(parts: &[engine::Part<'_>], model: &Model, offset: Option<usize>) -> (f32, f32) {
    offset
        .and_then(|row| engine::anchor(parts, model, row))
        .map_or((0.0, 0.0), |anchor| (anchor.x as f32, anchor.y as f32))
}

fn shift(frames: &mut [FrameData], (x, y): (f32, f32)) {
    for frame in frames {
        for corner in frame.vertices.as_chunks_mut::<2>().0 {
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

    fn center(frames: &[FrameData], index: usize) -> (f32, f32) {
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

        assert_eq!(center(&resting, 0), (0.0, -32.0));
        assert_eq!(center(&turned, 0), (0.0, 32.0));
    }

    #[test]
    fn the_row_is_measured_against_the_part_it_names() {
        let parts = vec![box_part(-1, 0, 0), box_part(0, 100, 0)];

        let root = resolve_frame(&rig(parts.clone(), vec![align(0)]), None, 0, Some(0));
        let child = resolve_frame(&rig(parts, vec![align(1)]), None, 0, Some(0));

        assert_eq!(center(&root, 1), (100.0, -32.0));
        assert_eq!(center(&child, 1), (0.0, -32.0));
    }

    #[test]
    fn the_row_subtracts_the_animated_pivot() {
        let rig = rig(vec![box_part(-1, 0, 0)], vec![align(0)]);
        let anim = animation(vec![drive(0, 6, hold(10))]);

        let frames = resolve_frame(&rig, Some(&anim), 0, Some(0));

        assert_eq!(center(&frames, 0), (0.0, -32.0));
    }

    #[test]
    fn the_row_is_carried_by_the_animated_scale() {
        let rig = rig(vec![box_part(-1, 0, 0)], vec![align(0)]);
        let anim = animation(vec![drive(0, 10, hold(2000))]);

        let frames = resolve_frame(&rig, Some(&anim), 0, Some(0));

        assert_eq!(center(&frames, 0), (0.0, -64.0));
    }
}
