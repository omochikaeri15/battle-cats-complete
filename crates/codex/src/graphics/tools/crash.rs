use crate::graphics::rig::{AnimModification, Animation, Keyframe, Model, ALIGNED_VERSION};

const EASE_POLYNOMIAL: i32 = 3;

const NOT_DRAWN: i32 = -1;

const ENDLESS: i32 = -1;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Encoding {
    Webp,
    Jpeg,
    Gif,
    Bmp,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Enemy,
    Cat,
    Either,
}

impl Side {
    fn rows(self) -> &'static [usize] {
        match self {
            Self::Enemy => &[0],
            Self::Cat | Self::Either => &[0, 1],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Fault {
    ScaleUnit,
    OpacityUnit,
    PolynomialTie {
        first: usize,
        second: usize,
        frame: i32,
    },
    ForeignSheet {
        id: i32,
    },
    AttackLength,
    EndlessAttack,
    MissingAlignment {
        row: usize,
    },
    StrayAlignment {
        row: usize,
        part: i32,
    },
    ForeignImage {
        found: Encoding,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sited {
    pub fault: Fault,
    pub part: Option<usize>,
    pub track: Option<usize>,
}

pub fn model_faults(model: &Model) -> Vec<Sited> {
    let mut faults = Vec::new();

    if model.parts.is_empty() { return faults; }

    let whole_file = |fault| Sited { fault, part: None, track: None };

    if model.scale_unit == 0 { faults.push(whole_file(Fault::ScaleUnit)); }
    if model.opacity_unit == 0 { faults.push(whole_file(Fault::OpacityUnit)); }

    faults
}

pub fn sheet_faults(model: &Model, unit: i32) -> Vec<Sited> {
    model.parts.iter()
        .enumerate()
        .filter(|(_, part)| part.id != NOT_DRAWN && part.id != unit)
        .map(|(index, part)| Sited {
            fault: Fault::ForeignSheet { id: part.id },
            part: Some(index),
            track: None,
        })
        .collect()
}

pub fn alignment_faults(model: &Model, side: Side) -> Vec<Sited> {
    let whole_file = |fault| Sited { fault, part: None, track: None };
    let rows = if model.version >= ALIGNED_VERSION { model.alignment.as_slice() } else { &[] };

    let missing = side.rows().iter()
        .filter(|row| **row >= rows.len())
        .map(|row| whole_file(Fault::MissingAlignment { row: *row }));

    let stray = rows.iter()
        .enumerate()
        .filter(|(_, align)| !usize::try_from(align.part).is_ok_and(|part| part < model.parts.len()))
        .map(|(row, align)| whole_file(Fault::StrayAlignment { row, part: align.part }));

    missing.chain(stray).collect()
}

pub fn anim_faults(anim: &Animation, model: &Model) -> Vec<Sited> {
    let mut faults = Vec::new();

    for (track, modification) in anim.modifications.iter().enumerate() {
        let part = usize::try_from(modification.part).ok()
            .filter(|index| *index < model.parts.len());

        for (first, second, frame) in ties(modification) {
            faults.push(Sited {
                fault: Fault::PolynomialTie { first, second, frame },
                part,
                track: Some(track),
            });
        }
    }

    faults
}

pub fn attack_faults(attack: &Animation, side: Side) -> Vec<Sited> {
    let whole_file = |fault| vec![Sited { fault, part: None, track: None }];

    match (side, attack.length()) {
        (Side::Enemy | Side::Either, ENDLESS) if attack.last_frame() == 0 => whole_file(Fault::EndlessAttack),
        (_, 0) => whole_file(Fault::AttackLength),
        _ => Vec::new(),
    }
}

pub fn image_faults(bytes: &[u8]) -> Vec<Sited> {
    if bytes.starts_with(&PNG_SIGNATURE) {
        return Vec::new();
    }

    let found = match bytes {
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Encoding::Webp,
        [0xff, 0xd8, 0xff, ..] => Encoding::Jpeg,
        [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Encoding::Gif,
        [b'B', b'M', ..] => Encoding::Bmp,
        _ => Encoding::Unknown,
    };

    vec![Sited { fault: Fault::ForeignImage { found }, part: None, track: None }]
}

fn ties(modification: &AnimModification) -> Vec<(usize, usize, i32)> {
    let keyframes = &modification.keyframes;
    let Some((earliest, latest)) = locals(modification) else { return Vec::new() };

    let mut found: Vec<(usize, usize, i32)> = Vec::new();

    for index in 0..keyframes.len().saturating_sub(1) {
        let (start, end) = (keyframes[index], keyframes[index + 1]);

        if start.ease != EASE_POLYNOMIAL { continue; }
        if start.frame >= end.frame || start.frame > latest || end.frame <= earliest { continue; }

        let (low, high) = run(keyframes, index);

        for outer in low..=high {
            for inner in (outer + 1)..=high {
                if keyframes[outer].frame != keyframes[inner].frame { continue; }

                let tie = (outer, inner, keyframes[outer].frame);
                if !found.contains(&tie) { found.push(tie); }
            }
        }
    }

    found
}

fn run(keyframes: &[Keyframe], index: usize) -> (usize, usize) {
    let mut low = index;
    while low > 0 && keyframes[low - 1].ease == EASE_POLYNOMIAL {
        low -= 1;
    }

    let mut high = index + 1;
    while high + 1 < keyframes.len() && keyframes[high].ease == EASE_POLYNOMIAL {
        high += 1;
    }

    (low, high)
}

fn locals(modification: &AnimModification) -> Option<(i32, i32)> {
    let first = modification.keyframes.first()?.frame;
    let last = modification.keyframes.last()?.frame;
    let span = last.wrapping_sub(first);
    let wraps = modification.loop_count == -1 || modification.loop_count > 0;

    match span {
        0 => None,
        span if span > 0 => Some((first, last.wrapping_sub(1))),
        span if wraps => Some((first, first.saturating_sub(span).saturating_sub(1))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::rig::{Alignment, ModelPart};
    use crate::graphics::tools::timeline;

    use super::*;

    fn keyframe(frame: i32, ease: i32) -> Keyframe {
        Keyframe { frame, value: frame, ease, ease_power: 0 }
    }

    fn curve(keys: &[(i32, i32)]) -> Animation {
        Animation {
            version: 1,
            modifications: vec![AnimModification {
                part: 0,
                kind: 4,
                loop_count: 1,
                keyframes: keys.iter().map(|(frame, ease)| keyframe(*frame, *ease)).collect(),
                ..AnimModification::default()
            }],
        }
    }

    fn model(parts: usize) -> Model {
        Model { parts: vec![ModelPart::default(); parts], ..Model::default() }
    }

    fn stamped(ids: &[i32]) -> Model {
        Model {
            parts: ids.iter().map(|id| ModelPart { id: *id, ..ModelPart::default() }).collect(),
            ..Model::default()
        }
    }

    fn aligned(version: i32, parts: usize, rows: &[i32]) -> Model {
        Model {
            version,
            alignment: rows.iter().map(|part| Alignment { part: *part, ..Alignment::default() }).collect(),
            ..model(parts)
        }
    }

    fn attack(loop_count: i32, frames: &[i32]) -> Animation {
        Animation {
            version: 1,
            modifications: vec![AnimModification {
                loop_count,
                keyframes: frames.iter().map(|frame| keyframe(*frame, 0)).collect(),
                ..AnimModification::default()
            }],
        }
    }

    fn ties_of(anim: &Animation) -> Vec<Fault> {
        anim_faults(anim, &model(1)).into_iter().map(|sited| sited.fault).collect()
    }

    #[test]
    fn a_zero_scale_or_opacity_column_is_caught_and_blamed_on_no_node() {
        let clean = Model { scale_unit: 1000, opacity_unit: 1000, ..model(1) };
        assert_eq!(model_faults(&clean), Vec::new());

        let scale = Model { scale_unit: 0, ..model(1) };
        assert_eq!(
            model_faults(&scale),
            vec![Sited { fault: Fault::ScaleUnit, part: None, track: None }],
        );

        let opacity = Model { opacity_unit: 0, ..model(1) };
        assert_eq!(
            model_faults(&opacity),
            vec![Sited { fault: Fault::OpacityUnit, part: None, track: None }],
        );

        let both = Model { scale_unit: 0, opacity_unit: 0, ..model(1) };
        assert_eq!(model_faults(&both).len(), 2);
    }

    #[test]
    fn a_zero_angle_column_is_not_a_fault() {
        let turned = Model { angle_unit: 0, ..model(1) };

        assert_eq!(model_faults(&turned), Vec::new());
    }

    #[test]
    fn a_model_with_no_parts_reaches_no_division() {
        let empty = Model { scale_unit: 0, opacity_unit: 0, ..model(0) };

        assert_eq!(model_faults(&empty), Vec::new());
    }

    #[test]
    fn a_rig_stamped_for_another_unit_is_caught_part_by_part() {
        let borrowed = stamped(&[-1, 34, -1, 34]);

        assert_eq!(
            sheet_faults(&borrowed, 0),
            vec![
                Sited { fault: Fault::ForeignSheet { id: 34 }, part: Some(1), track: None },
                Sited { fault: Fault::ForeignSheet { id: 34 }, part: Some(3), track: None },
            ],
        );

        assert_eq!(sheet_faults(&borrowed, 34), Vec::new());
        assert_eq!(sheet_faults(&stamped(&[-1, -1]), 7), Vec::new());
    }

    #[test]
    fn an_attack_animation_measuring_no_frames_is_caught_on_either_side() {
        let bare = Animation::default();
        let keyless = attack(1, &[]);

        for side in [Side::Enemy, Side::Cat, Side::Either] {
            assert_eq!(
                attack_faults(&bare, side),
                vec![Sited { fault: Fault::AttackLength, part: None, track: None }],
                "{side:?}",
            );
            assert_eq!(attack_faults(&keyless, side), attack_faults(&bare, side), "{side:?}");
        }
    }

    #[test]
    fn an_endless_attack_resting_on_frame_zero_is_caught_for_an_enemy_alone() {
        let stuck = attack(-1, &[0, 0]);

        assert_eq!(stuck.length(), -1);
        assert_eq!(
            attack_faults(&stuck, Side::Enemy),
            vec![Sited { fault: Fault::EndlessAttack, part: None, track: None }],
        );
        assert_eq!(attack_faults(&stuck, Side::Cat), Vec::new());
    }

    #[test]
    fn an_attack_animation_the_counter_can_advance_is_clean() {
        for side in [Side::Enemy, Side::Cat, Side::Either] {
            assert_eq!(attack_faults(&attack(-1, &[0, 30]), side), Vec::new(), "{side:?}");
            assert_eq!(attack_faults(&attack(1, &[0, 24]), side), Vec::new(), "{side:?}");
        }
    }

    #[test]
    fn a_side_reaching_past_the_alignment_block_is_caught_row_by_row() {
        let none = aligned(3, 2, &[]);

        assert_eq!(
            alignment_faults(&none, Side::Enemy),
            vec![Sited { fault: Fault::MissingAlignment { row: 0 }, part: None, track: None }],
        );
        assert_eq!(
            alignment_faults(&none, Side::Cat),
            vec![
                Sited { fault: Fault::MissingAlignment { row: 0 }, part: None, track: None },
                Sited { fault: Fault::MissingAlignment { row: 1 }, part: None, track: None },
            ],
        );

        let one = aligned(3, 2, &[0]);

        assert_eq!(alignment_faults(&one, Side::Enemy), Vec::new());
        assert_eq!(
            alignment_faults(&one, Side::Cat),
            vec![Sited { fault: Fault::MissingAlignment { row: 1 }, part: None, track: None }],
        );

        assert_eq!(alignment_faults(&aligned(3, 2, &[0, 1]), Side::Cat), Vec::new());
    }

    #[test]
    fn a_rig_of_no_stated_side_reports_what_either_side_would_reach() {
        let stuck = attack(-1, &[0, 0]);

        assert_eq!(attack_faults(&stuck, Side::Cat), Vec::new());
        assert_eq!(
            attack_faults(&stuck, Side::Either),
            vec![Sited { fault: Fault::EndlessAttack, part: None, track: None }],
        );

        let enemy = aligned(3, 2, &[0]);

        assert_eq!(alignment_faults(&enemy, Side::Enemy), Vec::new());
        assert_eq!(
            alignment_faults(&enemy, Side::Either),
            vec![Sited { fault: Fault::MissingAlignment { row: 1 }, part: None, track: None }],
        );

        assert_eq!(alignment_faults(&aligned(3, 2, &[0, 1]), Side::Either), Vec::new());
    }

    #[test]
    fn a_model_below_the_third_revision_carries_no_alignment_block() {
        let rows = aligned(2, 2, &[0, 1]);

        assert_eq!(
            alignment_faults(&rows, Side::Enemy),
            vec![Sited { fault: Fault::MissingAlignment { row: 0 }, part: None, track: None }],
        );
    }

    #[test]
    fn an_alignment_row_naming_a_part_the_model_lacks_is_caught_wherever_it_sits() {
        let stray = aligned(3, 2, &[0, 5, -1]);

        assert_eq!(
            alignment_faults(&stray, Side::Enemy),
            vec![
                Sited { fault: Fault::StrayAlignment { row: 1, part: 5 }, part: None, track: None },
                Sited { fault: Fault::StrayAlignment { row: 2, part: -1 }, part: None, track: None },
            ],
        );
    }

    #[test]
    fn two_polynomial_keys_on_one_frame_are_caught_and_blamed_on_their_track() {
        let held = curve(&[(0, 3), (10, 3), (10, 3), (20, 0)]);

        assert_eq!(
            anim_faults(&held, &model(1)),
            vec![Sited {
                fault: Fault::PolynomialTie { first: 1, second: 2, frame: 10 },
                part: Some(0),
                track: Some(0),
            }],
        );
    }

    #[test]
    fn a_tie_reachable_from_several_keys_of_one_run_is_reported_once() {
        let held = curve(&[(0, 3), (10, 3), (10, 3), (20, 3), (30, 0)]);

        assert_eq!(ties_of(&held), vec![Fault::PolynomialTie { first: 1, second: 2, frame: 10 }]);
    }

    #[test]
    fn a_shared_frame_outside_a_polynomial_run_is_left_alone() {
        for ease in [0, 1, 2] {
            let shared = curve(&[(0, ease), (10, ease), (10, ease), (20, ease)]);

            assert_eq!(ties_of(&shared), Vec::new(), "ease {ease}");
        }
    }

    #[test]
    fn a_polynomial_run_with_distinct_frames_is_clean() {
        assert_eq!(ties_of(&curve(&[(0, 3), (10, 3), (20, 3), (30, 0)])), Vec::new());
    }

    #[test]
    fn a_run_the_engine_never_evaluates_is_left_alone() {
        let held = curve(&[(7, 3), (7, 3), (7, 3)]);

        assert_eq!(timeline::value(&held.modifications[0], 9), Some(7));
        assert_eq!(ties_of(&held), Vec::new());
    }

    #[test]
    fn a_run_reached_only_past_the_last_keyframe_is_left_alone() {
        let backwards = Animation {
            version: 1,
            modifications: vec![AnimModification {
                loop_count: 0,
                keyframes: vec![keyframe(20, 3), keyframe(20, 3), keyframe(4, 3)],
                ..AnimModification::default()
            }],
        };

        assert_eq!(ties_of(&backwards), Vec::new());
    }

    #[test]
    fn a_modification_naming_a_part_the_model_lacks_is_still_reported() {
        let mut held = curve(&[(0, 3), (10, 3), (10, 3), (20, 0)]);
        held.modifications[0].part = 40;

        let faults = anim_faults(&held, &model(2));

        assert_eq!(faults.len(), 1);
        assert_eq!(faults[0].part, None);
        assert_eq!(faults[0].track, Some(0));
    }

    #[test]
    fn the_engine_survives_every_tie_this_reports() {
        let held = curve(&[(0, 3), (10, 3), (10, 3), (20, 0)]);

        assert!(!anim_faults(&held, &model(1)).is_empty());

        for frame in 0..20 {
            assert!(timeline::value(&held.modifications[0], frame).is_some(), "frame {frame}");
        }
    }

    #[test]
    fn only_a_png_signature_passes_as_a_sheet() {
        let fault = |found| vec![Sited { fault: Fault::ForeignImage { found }, part: None, track: None }];

        assert_eq!(image_faults(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 13]), Vec::new());
        assert_eq!(image_faults(b"RIFF\x10\0\0\0WEBPVP8 "), fault(Encoding::Webp));
        assert_eq!(image_faults(&[0xff, 0xd8, 0xff, 0xe0]), fault(Encoding::Jpeg));
        assert_eq!(image_faults(b"GIF89a"), fault(Encoding::Gif));
        assert_eq!(image_faults(b"BM6"), fault(Encoding::Bmp));
        assert_eq!(image_faults(b""), fault(Encoding::Unknown));
    }
}
