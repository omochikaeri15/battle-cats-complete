use serde::Serialize;

use crate::common::columns::{self, Column};
use crate::common::stream::{declared_rows, Reader};
use crate::graphics::tools::math;

use super::RigError;

/// The first format revision whose keyframe rows carry an easing exponent.
const POWERED_VERSION: i32 = 1;

/// A single control point on a modification's curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Keyframe {
    /// The frame at which this control point takes effect.
    pub frame: i32,
    /// The raw value the curve holds here, in the units of the modified property.
    pub value: i32,
    /// The interpolation applied between this control point and the next.
    pub ease: i32,
    /// The exponent the exponential easing raises its progress to.
    pub ease_power: i32,
}

impl Keyframe {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.maanim` keyframe row
    /// from the parser's own table instead of restating it.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        frame      : 0;
        value      : 1;
        ease       : 2;
        ease_power : 3;
    };
}

/// A single animated property of one model part over time.
///
/// A part that moves, rotates, and fades at once is described by three separate
/// modifications sharing a part index.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AnimModification {
    /// The index of the model part this drives, which the engine does not bound check.
    pub part: i32,
    /// The property being driven, from zero for the parent index to fourteen for the vertical flip.
    pub kind: i32,
    /// How many times the curve replays, where minus one replays it forever.
    pub loop_count: i32,
    /// The lower value bound the authoring tool recorded, which the engine ignores.
    pub min_value: i32,
    /// The upper value bound the authoring tool recorded, which the engine ignores.
    pub max_value: i32,
    /// The modification's declared name.
    pub name: String,
    /// The control points defining the curve.
    pub keyframes: Vec<Keyframe>,
}

impl AnimModification {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `.maanim` modification
    /// header from the parser's own table instead of restating it.
    /// [`AnimModification::name`] is the row's trailing text rather than a
    /// column, and [`AnimModification::keyframes`] follows on later rows.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        part       : 0;
        kind       : 1;
        loop_count : 2;
        min_value  : 3;
        max_value  : 4;
    };
}

/// A complete animation timeline for a unit's rig.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// The declared format version.
    pub version: i32,
    /// The property curves that make up the animation.
    pub modifications: Vec<AnimModification>,
}

impl Animation {
    /// Parses a `.maanim` byte stream into a structured animation timeline.
    ///
    /// The first line is discarded unread, the second names the version and the
    /// third the modification count. Each modification is a header row followed
    /// by its keyframe count and that many keyframe rows, whose fourth column is
    /// read only from the first format revision onwards.
    ///
    /// Exactly as many rows are read as each count declares, so a file that runs
    /// out repeats the last row it managed to read rather than stopping short.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the unit's `.maanim` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Animation` on success, or a `RigError`
    /// if the file contained no readable lines or declared a count the engine
    /// could not allocate.
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Result<Self, RigError> {
        let content = String::from_utf8_lossy(bytes);
        let mut reader = Reader::new(&content);

        if reader.line().is_none() { return Err(RigError::EmptyFile); }

        reader.row();
        let version = reader.value(0);

        reader.row();
        let count = declared_rows(reader.value(0)).ok_or(RigError::CountTooLarge)?;

        let mut modifications = Vec::with_capacity(count);

        for _ in 0..count {
            reader.row();

            let mut modification = AnimModification::default();
            let trailing = columns::apply(&reader.scalars(), AnimModification::COLUMNS, &mut modification);
            modification.name = reader.cells().get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            reader.row();
            let keys = declared_rows(reader.value(0)).ok_or(RigError::CountTooLarge)?;

            modification.keyframes = Vec::with_capacity(keys);

            for _ in 0..keys {
                reader.row();

                let mut keyframe = Keyframe::default();
                columns::apply(&reader.scalars(), Keyframe::COLUMNS, &mut keyframe);

                if version < POWERED_VERSION { keyframe.ease_power = 0; }

                modification.keyframes.push(keyframe);
            }

            modifications.push(modification);
        }

        Ok(Self { version, modifications })
    }

    /// Returns the number of frames the engine considers the animation to occupy.
    ///
    /// Every modification is measured across all of its replays and the longest
    /// one wins. A timeline holding any modification that replays forever has no
    /// length at all, and the engine reports minus one for it rather than a
    /// frame count.
    ///
    /// # Returns
    /// An `i32` containing the frame count, zero for a timeline with no usable
    /// modifications, or minus one for a timeline that never ends.
    pub fn length(&self) -> i32 {
        let mut longest = 0;

        for modification in &self.modifications {
            if modification.loop_count == -1 { return -1; }

            let (Some(first), Some(last)) = (modification.keyframes.first(), modification.keyframes.last()) else {
                continue;
            };

            let span = last.frame.wrapping_sub(first.frame);
            let played = modification.loop_count
                .wrapping_sub(1)
                .wrapping_mul(span)
                .wrapping_add(last.frame);

            if played >= longest { longest = played.wrapping_add(1); }
        }

        longest
    }

    /// Returns the interval after which every modification realigns with its own start.
    ///
    /// The engine wraps a replaying modification over the span between its first
    /// and last keyframe, so a timeline whose modifications all replay forever
    /// repeats over the least common multiple of those spans. A timeline holding
    /// any modification that stops has no such interval.
    ///
    /// # Returns
    /// An `Option` containing the repeat interval in frames, or `None` if any
    /// modification stops replaying or the combined interval overflows.
    pub fn period(&self) -> Option<i32> {
        let mut period = 1;

        for modification in &self.modifications {
            if modification.loop_count != -1 { return None; }

            let (Some(first), Some(last)) = (modification.keyframes.first(), modification.keyframes.last()) else {
                continue;
            };

            let span = last.frame - first.frame;
            if span <= 0 { continue; }

            period = (period / math::gcd(period, span)).checked_mul(span)?;
        }

        (period > 1).then_some(period)
    }

    /// Returns the number of frames the file itself defines.
    ///
    /// A timeline that ends occupies the length the engine measures across every
    /// replay. A timeline that never ends is measured by the furthest keyframe
    /// any modification reaches, which is the point a forever replaying
    /// modification wraps back to its first keyframe and so renders the same as
    /// frame zero, leaving the count one short of that frame.
    ///
    /// # Returns
    /// An `i32` containing the engine's length for a timeline that ends, and
    /// otherwise the furthest frame any modification reaches, which is at least
    /// one.
    pub fn declared_frames(&self) -> i32 {
        match self.length() {
            length if length > 0 => length,
            _ => self.last_frame().max(1),
        }
    }

    /// Returns the furthest frame any modification reaches.
    ///
    /// This is a frame index rather than a count, and is the last frame the file
    /// itself defines a control point for. It differs from
    /// [`Animation::declared_frames`] less one whenever a modification replays a
    /// fixed number of times, since the length the engine measures multiplies
    /// that modification's span by its replay count.
    ///
    /// # Returns
    /// An `i32` containing the index, or zero for a timeline with no keyframes.
    pub fn last_frame(&self) -> i32 {
        self.modifications.iter()
            .filter_map(|modification| modification.keyframes.last())
            .fold(0, |furthest, keyframe| furthest.max(keyframe.frame))
    }

    /// Returns the number of frames one full playback cycle occupies.
    ///
    /// Every replaying modification wraps over the span between its first and
    /// last keyframe, so a timeline realigns over the least common multiple of
    /// those spans. This is that interval, expressed as a frame count, rejected
    /// only where the file itself rules it out: a timeline holding a
    /// modification that plays exactly once never returns to its start, and an
    /// interval shorter than the content the file declares would truncate it. A
    /// timeline with no replaying span is a held pose and occupies the frames it
    /// declares.
    ///
    /// The interval is the exact least common multiple and is not bounded here,
    /// so a timeline whose spans share no factors can resolve to a cycle far
    /// longer than the content it was authored from.
    ///
    /// # Returns
    /// An `Option` containing the cycle length in frames, which is at least one,
    /// or `None` when the timeline does not return to its start.
    pub fn loop_frames(&self) -> Option<i32> {
        let last = self.last_frame();
        let mut cycle: i32 = 1;
        let mut repeating = false;

        for modification in &self.modifications {
            if modification.loop_count == 1 { return None; }

            let (Some(first), Some(final_key)) = (modification.keyframes.first(), modification.keyframes.last()) else {
                continue;
            };

            let span = final_key.frame - first.frame;
            if span <= 0 { continue; }

            cycle = (cycle / math::gcd(cycle, span)).checked_mul(span)?;
            repeating = true;
        }

        if !repeating { return Some(last + 1); }

        (cycle >= last).then_some(cycle)
    }

    /// Returns the number of frames to play before returning to the start.
    ///
    /// The engine never loops an animation itself: a modification that replays
    /// wraps its own keyframe range, and [`Animation::length`] reports minus one
    /// for a timeline that never ends. Anything that plays a timeline back still
    /// needs somewhere to restart, so this takes the playback cycle where the
    /// timeline has one and falls back to [`Animation::declared_frames`]. That
    /// cycle is the least common multiple of the replaying spans and is not
    /// bounded, so anything sizing a control, a default range or a bounded sweep
    /// wants [`Animation::declared_frames`] instead.
    ///
    /// # Returns
    /// An `i32` containing the frame count, which is at least one.
    pub fn playback_frames(&self) -> i32 {
        self.loop_frames().unwrap_or_else(|| self.declared_frames())
    }

    /// Measures an animation's length without building the full timeline.
    ///
    /// This walks the modification headers and their first and last keyframe
    /// rows directly, and is substantially cheaper than [`Animation::parse`]
    /// followed by [`Animation::length`], which it agrees with.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the unit's `.maanim` file.
    ///
    /// # Returns
    /// An `Option` containing the animation's length in frames, minus one for a
    /// timeline that never ends, or `None` if the file contained no readable
    /// lines or declared a count the engine could not allocate.
    pub fn scan_length(bytes: impl AsRef<[u8]>) -> Option<i32> {
        Self::scan_length_inner(bytes.as_ref())
    }

    fn scan_length_inner(bytes: &[u8]) -> Option<i32> {
        let content = String::from_utf8_lossy(bytes);
        let mut reader = Reader::new(&content);

        reader.line()?;

        reader.row();
        reader.row();
        let count = declared_rows(reader.value(0))?;

        let mut longest = 0;

        for _ in 0..count {
            reader.row();
            let loop_count = reader.value(2);

            if loop_count == -1 { return Some(-1); }

            reader.row();
            let keys = declared_rows(reader.value(0))?;

            let (mut first, mut last) = (0, 0);

            for index in 0..keys {
                reader.row();

                last = reader.value(0);
                if index == 0 { first = last; }
            }

            if keys > 0 {
                let played = loop_count
                    .wrapping_sub(1)
                    .wrapping_mul(last.wrapping_sub(first))
                    .wrapping_add(last);

                if played >= longest { longest = played.wrapping_add(1); }
            }
        }

        Some(longest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(version: i32, tail: &[&str]) -> String {
        let mut lines = vec!["[modelanim:animation]".to_string(), version.to_string()];
        lines.extend(tail.iter().map(|line| (*line).to_string()));

        lines.join("\n")
    }

    #[test]
    fn a_version_below_one_carries_no_easing_exponent() {
        let tail = ["1", "0,4,1,0,0,move", "1", "0,50,3,7"];

        assert_eq!(
            Animation::parse(file(0, &tail)).map(|anim| anim.modifications[0].keyframes[0].ease_power),
            Ok(0),
        );
        assert_eq!(
            Animation::parse(file(1, &tail)).map(|anim| anim.modifications[0].keyframes[0].ease_power),
            Ok(7),
        );
    }

    #[test]
    fn a_blank_line_is_a_row_like_any_other() {
        let anim = Animation::parse(file(1, &["1", "", "1", "5,50,0,0"])).expect("a timeline parses");

        assert_eq!(anim.modifications[0].part, 0);
        assert_eq!(anim.modifications[0].loop_count, 0);
        assert_eq!(anim.modifications[0].keyframes.len(), 1);
    }

    #[test]
    fn a_file_that_runs_out_repeats_the_last_row_it_read() {
        let truncated = file(1, &["1", "0,4,1,0,0,move", "3", "5,50,0,0"]);
        let anim = Animation::parse(&truncated).expect("a timeline parses");

        let keyframes = &anim.modifications[0].keyframes;

        assert_eq!(keyframes.len(), 3);
        assert_eq!(keyframes[1], keyframes[0]);
        assert_eq!(keyframes[2], keyframes[0]);
        assert_eq!(Animation::scan_length(&truncated), Some(anim.length()));
    }

    #[test]
    fn a_count_the_engine_aborts_on_is_refused() {
        assert_eq!(Animation::parse(file(1, &["-1"])), Err(RigError::CountTooLarge));
        assert_eq!(Animation::parse(file(1, &["1", "0,4,1,0,0", "-2"])), Err(RigError::CountTooLarge));
        assert_eq!(Animation::parse(""), Err(RigError::EmptyFile));
    }

    #[test]
    fn a_declared_modification_survives_holding_no_keyframes() {
        let anim = Animation::parse(file(1, &["2", "0,4,1,0,0", "0", "1,4,-1,0,0", "0"]))
            .expect("a timeline parses");

        assert_eq!(anim.modifications.len(), 2);
        assert_eq!(anim.length(), -1);
    }

    fn modification(loop_count: i32, frames: &[i32]) -> AnimModification {
        AnimModification {
            loop_count,
            keyframes: frames.iter().map(|&frame| Keyframe { frame, ..Keyframe::default() }).collect(),
            ..AnimModification::default()
        }
    }

    #[test]
    fn keyframe_columns_map_one_field_each() {
        columns::assert_one_field_per_column(Keyframe::COLUMNS);
    }

    #[test]
    fn modification_columns_map_one_field_each() {
        columns::assert_one_field_per_column(AnimModification::COLUMNS);
    }

    #[test]
    fn length_counts_every_replay() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(3, &[2, 12]), modification(1, &[0, 5])],
        };

        assert_eq!(animation.length(), 33);
    }

    #[test]
    fn an_endless_timeline_has_no_length() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(4, &[0, 30]), modification(-1, &[0, 8])],
        };

        assert_eq!(animation.length(), -1);
    }

    #[test]
    fn period_is_the_common_multiple_of_endless_spans() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(-1, &[0, 12])],
        };

        assert_eq!(animation.period(), Some(24));
    }

    #[test]
    fn declared_frames_ignores_the_realignment_interval() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(-1, &[0, 140])],
        };

        assert_eq!(animation.playback_frames(), 280);
        assert_eq!(animation.declared_frames(), 140);
    }

    #[test]
    fn declared_frames_counts_a_timeline_that_ends() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(1, &[0, 152])],
        };

        assert_eq!(animation.declared_frames(), 153);
    }

    #[test]
    fn loop_frames_counts_the_realignment_interval_without_padding() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(-1, &[0, 12])],
        };

        assert_eq!(animation.loop_frames(), Some(24));
    }

    #[test]
    fn loop_frames_refuses_an_interval_that_truncates_the_content() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 50]), modification(-1, &[84, 84])],
        };

        assert_eq!(animation.loop_frames(), None);
        assert_eq!(animation.playback_frames(), animation.declared_frames());
        assert_eq!(animation.declared_frames(), 84);
    }

    #[test]
    fn loop_frames_reports_an_interval_no_player_would_use() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 9_973]), modification(-1, &[0, 9_967])],
        };

        assert_eq!(animation.loop_frames(), Some(99_400_891));
    }

    #[test]
    fn a_play_once_modification_has_no_cycle() {
        let animation = Animation { version: 1, modifications: vec![modification(1, &[0, 129])] };

        assert_eq!(animation.loop_frames(), None);
        assert_eq!(animation.playback_frames(), 130);
    }

    #[test]
    fn a_held_pose_occupies_the_frames_it_declares() {
        let animation = Animation { version: 1, modifications: vec![modification(-1, &[7, 7])] };

        assert_eq!(animation.loop_frames(), Some(8));
    }

    #[test]
    fn last_frame_is_not_the_declared_count_less_one() {
        let animation = Animation { version: 1, modifications: vec![modification(3, &[2, 12])] };

        assert_eq!(animation.last_frame(), 12);
        assert_eq!(animation.declared_frames(), 33);
    }

    #[test]
    fn period_rejects_a_timeline_that_stops() {
        let animation = Animation {
            version: 1,
            modifications: vec![modification(-1, &[0, 8]), modification(1, &[0, 12])],
        };

        assert_eq!(animation.period(), None);
    }
}
