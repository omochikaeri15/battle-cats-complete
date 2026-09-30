//! The rig states on which the game's own animation pass faults.
//!
//! Each one is a place where this crate deliberately guards an operation the
//! engine leaves unguarded, so a rig that resolves cleanly here can still kill
//! the game: `engine::over_unit` returns the value untouched where the engine
//! divides by a zero unit column, `engine::polynomial` skips a term where the
//! engine divides by the gap between two keyframes naming one frame, and
//! `engine::deploy` reads a rig's single sprite sheet where the engine indexes
//! an array of them by the part's resolved identifier. Two more sit outside the
//! frame pass entirely: `animate::shift` leaves a rig alone where its alignment
//! block does not reach the row asked for and the engine dereferences that row
//! regardless, and nothing here advances an entity's frame counter at all where
//! the engine takes the next frame modulo an animation length it never checks.
//! Nothing here changes how a frame resolves; it reports where the two
//! disagree.
//!
//! Only the states the decompilation settles are reported. A shape the engine
//! guards is not a fault however malformed it looks: a modification whose first
//! and last keyframe share a frame, one holding no keyframes at all, a part
//! index past the model's part count, a sprite index outside the cut list, and a
//! parent cycle are all handled by the engine and are absent from this module by
//! design.

use crate::graphics::rig::{AnimModification, Animation, Keyframe, Model, ALIGNED_VERSION};

/// The interpolation whose term the engine divides by the gap between two keyframes.
const EASE_POLYNOMIAL: i32 = 3;

/// The identifier that marks a part the engine never draws.
const NOT_DRAWN: i32 = -1;

/// The length the engine reports for an animation that never ends.
const ENDLESS: i32 = -1;

/// The entity map a rig is installed into.
///
/// The engine keeps one animation map and one alignment index per side, so the
/// same files reach a different division depending on which of the two a rig is
/// drawn as. A caller that cannot tell the two apart states [`Side::Either`],
/// which reports every state either of them faults on and so reports a rig only
/// the other side would have crashed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// The enemy side, whose attacking frame counter falls back to the furthest keyframe and whose placement reads the first alignment row.
    Enemy,
    /// The cat side, covering forms and souls, whose attacking frame counter has no fallback and whose placement reads the second alignment row as well as the first.
    Cat,
    /// Neither side in particular, which reports what the two of them reach together.
    Either,
}

impl Side {
    /// The alignment rows the engine indexes for a rig on this side.
    fn rows(self) -> &'static [usize] {
        match self {
            Self::Enemy => &[0],
            Self::Cat | Self::Either => &[0, 1],
        }
    }
}

/// A state in which the game's own animation pass faults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Fault {
    /// The model's scale divisor is zero, which the placement pass divides by once for a root part and twice for every other.
    ScaleUnit,
    /// The model's opacity divisor is zero, which the placement pass divides by alongside the scale.
    OpacityUnit,
    /// Two keyframes of one polynomial run name the same frame, and the run divides by the gap between them.
    PolynomialTie {
        /// The index of the earlier keyframe within the modification.
        first: usize,
        /// The index of the later keyframe within the modification.
        second: usize,
        /// The frame both keyframes name.
        frame: i32,
    },
    /// A part draws from a sheet the entity being drawn does not hold, which the draw pass dereferences as a null pointer.
    ForeignSheet {
        /// The identifier the part's second column names.
        id: i32,
    },
    /// The attack animation measures no frames, which the frame counter takes the next frame modulo.
    AttackLength,
    /// The attack animation never ends and reaches no frame past zero, which an attacking enemy's frame counter takes the next frame modulo instead.
    EndlessAttack,
    /// An alignment row the side's placement reads is absent, which it dereferences as a null pointer.
    MissingAlignment {
        /// The index of the row the engine reads.
        row: usize,
    },
    /// An alignment row names a part the model does not hold, which the loader caches as a pointer past the part array and the placement dereferences.
    StrayAlignment {
        /// The index of the alignment row.
        row: usize,
        /// The part index the row's first column names.
        part: i32,
    },
}

/// A fault together with the node of the rig it belongs to.
///
/// A fault belonging to the whole file leaves both fields empty, which is how a
/// consumer tells it apart from one it can attribute to a tree row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sited {
    /// The state the game faults on.
    pub fault: Fault,
    /// The index of the model part the fault belongs to, or `None` where no part carries it.
    pub part: Option<usize>,
    /// The index of the modification the fault belongs to, or `None` where no modification drives it.
    pub track: Option<usize>,
}

/// Reports the faults a model carries on its own, independent of any animation.
///
/// The placement pass divides every part's scale and opacity by the model's unit
/// columns before it decides whether the part is drawn, so a zero in either
/// column faults on the first part placed. A model declaring no parts never
/// reaches the division. The angle column is divided in single precision and
/// yields an infinity rather than a fault, so a zero there is not reported.
///
/// # Arguments
/// * `model` - The parsed part hierarchy to inspect.
///
/// # Returns
/// A `Vec<Sited>` holding one entry per zero unit column, each sited on neither
/// a part nor a modification, and empty for a model the engine can place.
pub fn model_faults(model: &Model) -> Vec<Sited> {
    let mut faults = Vec::new();

    if model.parts.is_empty() { return faults; }

    let whole_file = |fault| Sited { fault, part: None, track: None };

    if model.scale_unit == 0 { faults.push(whole_file(Fault::ScaleUnit)); }
    if model.opacity_unit == 0 { faults.push(whole_file(Fault::OpacityUnit)); }

    faults
}

/// Reports the parts drawing from a sheet the entity being drawn does not hold.
///
/// The engine holds an array of loaded sheets and indexes it by the part's
/// identifier, while a rig installed on one entity slot holds only that entity's
/// sheet, so a part naming another identifier reads a null pointer and
/// dereferences it. A part the engine never draws names no sheet and is never a
/// fault.
///
/// A model whose parts genuinely span several identifiers is only faulty where
/// it is being installed onto a single slot, which this cannot know, so the
/// caller supplies the identifier the rig is being drawn as.
///
/// # Arguments
/// * `model` - The parsed part hierarchy to inspect.
/// * `unit` - The identifier of the entity slot the rig is drawn as.
///
/// # Returns
/// A `Vec<Sited>` holding one entry per offending part, each sited on that part,
/// and empty for a model whose drawn parts all name `unit`.
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

/// Reports the faults a model's alignment block carries on the side it is drawn as.
///
/// The draw pass reads the first alignment row for every entity it puts on
/// screen and the bounding pass reads the second for a cat, neither of them
/// checking the index against the rows the model declares, so a row the side
/// reaches past is dereferenced as a null pointer. A model below the third
/// format revision carries no alignment block at all whatever rows trail its
/// units row, and reaches that null on the first row.
///
/// Each row caches a pointer to the part its first column names, computed as an
/// offset into the part array when the model loads and bounds checked at
/// neither end, so a row naming a part the model does not hold is dereferenced
/// as a pointer past that array. Every row carries one, so a row is reported
/// wherever it sits rather than only where this side reads it.
///
/// A rig of no stated side is held to both rows, so a model carrying the single
/// row an enemy needs is reported for the second row a cat would have read.
///
/// # Arguments
/// * `model` - The parsed part hierarchy to inspect.
/// * `side` - The entity map the rig is installed into.
///
/// # Returns
/// A `Vec<Sited>` holding one entry per row the side reads and the model lacks
/// followed by one per row naming a part outside the model, each sited on
/// neither a part nor a modification, and empty for a model the engine can
/// align.
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

/// Reports the faults an animation carries when played against a model.
///
/// Only the polynomial easing divides by the gap between two keyframes, and it
/// does so across the whole run of consecutive polynomial keyframes rather than
/// the bracketing pair alone, so two keyframes naming one frame are a fault
/// where a run reaches both and harmless anywhere else. The model is read only
/// to site each fault on the part its modification drives, and a modification
/// naming a part the model does not hold is still reported, because the engine
/// bounds checks the part index only after it has evaluated the curve.
///
/// # Arguments
/// * `anim` - The parsed animation timeline to inspect.
/// * `model` - The part hierarchy the animation is played against.
///
/// # Returns
/// A `Vec<Sited>` holding one entry per pair of tied keyframes, each sited on
/// its modification and on the part that modification drives, and empty for an
/// animation the engine can evaluate.
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

/// Reports the faults an attack animation carries on the side it is played on.
///
/// An attacking entity advances by taking its next frame modulo the animation's
/// length, which is unguarded, so an animation measuring no frames faults on the
/// first advance. An enemy whose attack animation never ends takes that frame
/// modulo the furthest frame any modification reaches instead, which is equally
/// unguarded, so an endless animation holding every modification at frame zero
/// faults there. A cat reaching the same state divides by minus one, which the
/// engine survives, so a rig of no stated side reports it where a cat does not.
///
/// The animation is the rig's third slot, which the engine files under `_e02`,
/// `_f02`, `_c02` and `_s02`. No other slot reaches either division.
///
/// # Arguments
/// * `attack` - The parsed animation occupying the rig's attack slot.
/// * `side` - The entity map the rig is installed into.
///
/// # Returns
/// A `Vec<Sited>` holding the one fault the animation carries, sited on neither
/// a part nor a modification, and empty for an animation the frame counter can
/// advance.
pub fn attack_faults(attack: &Animation, side: Side) -> Vec<Sited> {
    let whole_file = |fault| vec![Sited { fault, part: None, track: None }];

    match (side, attack.length()) {
        (Side::Enemy | Side::Either, ENDLESS) if attack.last_frame() == 0 => whole_file(Fault::EndlessAttack),
        (_, 0) => whole_file(Fault::AttackLength),
        _ => Vec::new(),
    }
}

/// Collects the keyframe pairs a reachable polynomial run divides a zero gap by.
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

/// The run of consecutive polynomial keyframes the engine accumulates a term for.
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

/// The inclusive range of wrapped frames the engine can reach the easing at.
///
/// A modification whose first and last keyframe share a frame never reaches the
/// easing at all, and one resting on its final keyframe never reaches it either,
/// which is why both leave nothing to report.
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
}
