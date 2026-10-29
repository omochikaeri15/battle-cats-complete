//! Where a single modification sits in its own timeline at a given frame.
//!
//! [`crate::graphics::animate::resolve_frame`] resolves every curve of an
//! animation at once and reports only the geometry that came out of them, so
//! nothing there says which keyframe drove a value. This module runs the
//! engine's own walk for one modification at a time and reports the keyframe it
//! lands on alongside the frame it resolved against.

use crate::graphics::engine;
use crate::graphics::rig::AnimModification;

/// Where a modification is in its own timeline at a given frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Playhead {
    /// The index of the keyframe whose span holds the frame.
    pub key: usize,
    /// The frame the modification resolves against after its own replay wrapping.
    pub local: i32,
}

/// Resolves which of a modification's keyframes drives it at a frame.
///
/// The engine scans the keyframes forward and takes the first pair enclosing the
/// resolved frame, so a file declaring its frames out of order lands on whichever
/// pair that scan reaches first.
///
/// # Arguments
/// * `modification` - The curve to resolve.
/// * `frame` - The frame to resolve at. The engine has no notion of a fractional frame.
///
/// # Returns
/// An `Option` containing the `Playhead` the frame resolves to, or `None` if the
/// modification holds no keyframes, the frame precedes its first keyframe, or no
/// pair of keyframes encloses the resolved frame.
pub fn playhead(modification: &AnimModification, frame: i32) -> Option<Playhead> {
    let keyframes = &modification.keyframes;
    let first = keyframes.first()?;
    let last = keyframes.last()?;

    if first.frame > frame { return None; }

    let span = last.frame.wrapping_sub(first.frame);

    if span == 0 { return Some(Playhead { key: 0, local: first.frame }); }

    let local = engine::local_frame(modification, first.frame, last.frame, span, frame);

    if local == last.frame {
        return Some(Playhead { key: keyframes.len() - 1, local });
    }

    let key = keyframes.windows(2)
        .position(|pair| local >= pair[0].frame && local < pair[1].frame)?;

    Some(Playhead { key, local })
}

/// Resolves the value a modification holds at a frame.
///
/// # Arguments
/// * `modification` - The curve to resolve.
/// * `frame` - The frame to resolve at. The engine has no notion of a fractional frame.
///
/// # Returns
/// An `Option` containing the raw value in the units of the modified property,
/// or `None` before the modification's first keyframe, where the engine leaves
/// the property alone entirely.
pub fn value(modification: &AnimModification, frame: i32) -> Option<i32> {
    engine::evaluate(modification, frame)
}

/// Folds a frame back into a modification's own keyframe range.
///
/// A modification replaying forever wraps over its span every time. One with a
/// replay count wraps only while replays remain and then rests on its final
/// keyframe, which is also where any other replay count lands immediately. A
/// modification whose keyframes all name one frame has no span to wrap over and
/// rests on that frame.
///
/// # Arguments
/// * `modification` - The curve to fold the frame into.
/// * `frame` - The frame to fold. The engine has no notion of a fractional frame.
///
/// # Returns
/// An `i32` holding the frame the modification resolves against, or `None` if
/// the modification holds no keyframes.
pub fn local_frame(modification: &AnimModification, frame: i32) -> Option<i32> {
    let first = modification.keyframes.first()?;
    let last = modification.keyframes.last()?;

    let span = last.frame.wrapping_sub(first.frame);

    if span == 0 && last.frame <= frame { return Some(last.frame); }

    Some(engine::local_frame(modification, first.frame, last.frame, span, frame))
}

#[cfg(test)]
mod tests {
    use crate::graphics::animate::resolve_frame;
    use crate::graphics::rig::{Animation, Keyframe, Model, ModelPart, Rig, SpriteCut, SpriteSheet};

    use super::*;

    const CUTS: i32 = 48;
    const SPRITE: i32 = 2;
    const SPAN: i32 = 20;

    fn keyframe(frame: i32, value: i32, ease: i32, ease_power: i32) -> Keyframe {
        Keyframe { frame, value, ease, ease_power }
    }

    fn curve(loop_count: i32) -> AnimModification {
        AnimModification {
            part: 0,
            kind: SPRITE,
            loop_count,
            keyframes: vec![
                keyframe(2, 10, 0, 0),
                keyframe(6, 30, 1, 0),
                keyframe(10, 20, 2, 2),
                keyframe(14, 40, 3, 0),
                keyframe(18, 12, 3, 0),
                keyframe(22, 25, 0, 0),
            ],
            ..AnimModification::default()
        }
    }

    fn flat() -> AnimModification {
        AnimModification {
            part: 0,
            kind: SPRITE,
            loop_count: -1,
            keyframes: vec![keyframe(5, 7, 0, 0), keyframe(5, 9, 0, 0)],
            ..AnimModification::default()
        }
    }

    fn rig() -> Rig {
        let model = Model {
            parts: vec![ModelPart {
                parent: -1,
                id: 0,
                sprite: 0,
                z: 0,
                scale_x: 1000,
                scale_y: 1000,
                opacity: 1000,
                ..ModelPart::default()
            }],
            ..Model::default()
        };

        let cuts = (0..CUTS)
            .map(|index| SpriteCut { x: index, y: 0, width: 10, height: 10, name: String::new() })
            .collect();

        Rig { model, sheet: SpriteSheet { cuts, ..SpriteSheet::default() } }
    }

    fn animation(modification: AnimModification) -> Animation {
        Animation { version: 1, modifications: vec![modification] }
    }

    #[test]
    fn the_value_agrees_with_the_whole_rig_pass() {
        let rig = rig();

        for loop_count in [-1, 0, 2] {
            let modification = curve(loop_count);
            let anim = animation(curve(loop_count));

            for frame in -4..64 {
                let expected = value(&modification, frame).unwrap_or(0);
                assert!((0..CUTS).contains(&expected), "the fixture left the atlas at frame {frame}");

                let drawn = resolve_frame(&rig, Some(&anim), frame, None)
                    .first()
                    .map(|entry| entry.sprite_index);

                assert_eq!(drawn, Some(expected as usize), "loop {loop_count} frame {frame}");
            }
        }
    }

    #[test]
    fn the_bracket_encloses_the_frame_it_resolved() {
        for loop_count in [-1, 0, 2] {
            let modification = curve(loop_count);

            for frame in -4..64 {
                let Some(head) = playhead(&modification, frame) else { continue };
                let Some(key) = modification.keyframes.get(head.key) else {
                    panic!("loop {loop_count} frame {frame} named a keyframe that does not exist");
                };

                assert!(key.frame <= head.local, "loop {loop_count} frame {frame}");

                if let Some(next) = modification.keyframes.get(head.key + 1) {
                    assert!(head.local < next.frame, "loop {loop_count} frame {frame}");
                }
            }
        }
    }

    #[test]
    fn a_keyframes_own_frame_lands_on_that_keyframe() {
        let modification = curve(0);

        for (index, key) in modification.keyframes.iter().enumerate() {
            let head = playhead(&modification, key.frame);

            assert_eq!(head.map(|head| head.key), Some(index), "keyframe {index}");
            assert_eq!(head.map(|head| head.local), Some(key.frame), "keyframe {index}");
        }
    }

    #[test]
    fn a_frame_before_the_first_keyframe_resolves_to_nothing() {
        let modification = curve(-1);

        for frame in [-8, 0, 1] {
            assert_eq!(playhead(&modification, frame), None, "frame {frame}");
            assert_eq!(value(&modification, frame), None, "frame {frame}");
            assert_eq!(local_frame(&modification, frame), Some(frame), "frame {frame}");
        }
    }

    #[test]
    fn a_curve_with_no_span_rests_on_its_first_keyframe() {
        let modification = flat();

        for frame in [5, 6, 400] {
            assert_eq!(playhead(&modification, frame), Some(Playhead { key: 0, local: 5 }), "frame {frame}");
            assert_eq!(value(&modification, frame), Some(7), "frame {frame}");
            assert_eq!(local_frame(&modification, frame), Some(5), "frame {frame}");
        }

        assert_eq!(playhead(&modification, 4), None);
        assert_eq!(local_frame(&modification, 4), Some(4));
    }

    #[test]
    fn a_curve_replaying_forever_wraps_onto_its_own_keys() {
        let modification = curve(-1);

        for frame in 2..22 {
            assert_eq!(playhead(&modification, frame), playhead(&modification, frame + SPAN), "frame {frame}");
            assert_eq!(value(&modification, frame), value(&modification, frame + SPAN), "frame {frame}");
        }

        assert_eq!(playhead(&modification, 22), Some(Playhead { key: 0, local: 2 }));
        assert_eq!(value(&modification, 22), Some(10));
    }

    #[test]
    fn a_curve_that_runs_out_of_replays_holds_its_last_key() {
        let modification = curve(2);

        assert_eq!(playhead(&modification, 41), Some(Playhead { key: 4, local: 21 }));

        for frame in [42, 43, 400] {
            assert_eq!(playhead(&modification, frame), Some(Playhead { key: 5, local: 22 }), "frame {frame}");
            assert_eq!(value(&modification, frame), Some(25), "frame {frame}");
            assert_eq!(local_frame(&modification, frame), Some(22), "frame {frame}");
        }
    }

    #[test]
    fn a_curve_with_no_keyframes_resolves_to_nothing() {
        let modification = AnimModification::default();

        assert_eq!(playhead(&modification, 0), None);
        assert_eq!(value(&modification, 0), None);
        assert_eq!(local_frame(&modification, 0), None);
    }
}
