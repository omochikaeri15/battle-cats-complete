//! Parsing and geometric analysis of a unit's animation rig.
//!
//! A unit's appearance is defined by three files: the texture atlas, its sprite
//! cut list, and the model hierarchy. This module turns them into a single
//! [`Rig`], and parses the separate animation timelines that drive it.

mod imgcut;
mod maanim;
mod mamodel;

use std::error;
use std::fmt;

use super::animate::{resolve_frame, FrameData};
use super::tools::{boundary, periodicity};

pub use boundary::{BoundingBox, Tolerance};
pub use imgcut::{Opaque, SpriteCut, SpriteSheet};
pub use maanim::{AnimModification, Animation, Keyframe};
pub use mamodel::{Alignment, Model, ModelPart};

pub(crate) use mamodel::ALIGNED_VERSION;

/// Represents errors that can occur while parsing a unit's rig or animations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RigError {
    /// The supplied bytes contained no lines at all.
    EmptyFile,
    /// The supplied bytes were too short to contain a complete file header.
    TruncatedHeader,
    /// No line declaring a usable model part count was found in the file's header.
    NoPartHeader,
    /// The texture atlas could not be decoded, and salvage of the stream also failed.
    ImageDecodeFailed,
    /// The atlas decoded successfully but its cut list described no usable sprite regions.
    NoSpriteCuts,
    /// A declared row count was negative or larger than the engine's own resize could allocate.
    CountTooLarge,
}

impl fmt::Display for RigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no lines at all."),
            Self::TruncatedHeader => write!(f, "The provided bytes were too short to contain a complete header."),
            Self::NoPartHeader => write!(f, "No usable model part count was declared in the file header."),
            Self::ImageDecodeFailed => write!(f, "The texture atlas could not be decoded or salvaged."),
            Self::NoSpriteCuts => write!(f, "The sprite cut list described no usable regions."),
            Self::CountTooLarge => write!(f, "A declared row count was negative or too large to allocate."),
        }
    }
}

impl error::Error for RigError {}

/// A unit's complete visual definition: its part hierarchy and its texture atlas.
///
/// The engine splits this across three files, because the model references
/// sprite regions by index and those indices are only meaningful against the
/// atlas that accompanies it.
#[derive(Debug, Clone)]
pub struct Rig {
    /// The hierarchical part tree describing how the unit is assembled and animated.
    pub model: Model,
    /// The texture atlas and the sprite regions the model's parts index into.
    pub sheet: SpriteSheet,
}

/// A detected repeating interval within an animation, measured in frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cycle {
    /// The first frame of the repeating interval.
    pub start: i32,
    /// The frame at which the interval repeats, one past the last distinct frame.
    pub end: i32,
}

impl Cycle {
    /// Returns the length of the interval in frames.
    ///
    /// # Returns
    /// An `i32` containing the number of frames between the start of the
    /// interval and its repetition.
    pub fn length(&self) -> i32 {
        self.end - self.start
    }
}

impl Rig {
    /// Parses a unit's texture atlas, sprite cut list, and model hierarchy into a rig.
    ///
    /// # Arguments
    /// * `png` - The raw bytes of the unit's PNG texture atlas.
    /// * `imgcut` - The raw bytes of the unit's `.imgcut` sprite region list.
    /// * `mamodel` - The raw bytes of the unit's `.mamodel` part hierarchy.
    ///
    /// # Returns
    /// A `Result` containing the assembled `Rig` on success, or a `RigError`
    /// identifying which of the three inputs could not be interpreted.
    pub fn parse(
        png: impl AsRef<[u8]>,
        imgcut: impl AsRef<[u8]>,
        mamodel: impl AsRef<[u8]>,
    ) -> Result<Self, RigError> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref(), mamodel.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8], mamodel: &[u8]) -> Result<Self, RigError> {
        let sheet = SpriteSheet::parse(png, imgcut)?;
        let model = Model::parse(mamodel)?;

        Ok(Self { model, sheet })
    }

    /// Calculates the smallest rectangle enclosing the rig across a set of animations.
    ///
    /// Frames are resolved and their visible geometry accumulated, so the result
    /// encloses the range of motion rather than any single pose. Parts the
    /// tolerance judges invisible are excluded.
    ///
    /// # Arguments
    /// * `animations` - The animations to sweep. An empty slice measures the rig in its resting pose.
    /// * `tolerance` - A value from zero to one controlling how aggressively marginal parts are discarded, where zero excludes nothing.
    /// * `to_frame` - The last frame to measure in each animation, or `None` to sweep every animation in full.
    /// * `offset` - The index of the alignment row the rig is placed by, or `None` to measure it at the engine's own origin.
    ///
    /// # Returns
    /// An `Option` containing the enclosing `BoundingBox`, or `None` if no part
    /// of the rig was visible in any evaluated frame.
    pub fn calculate_bounds(
        &self,
        animations: &[&Animation],
        tolerance: f32,
        to_frame: Option<i32>,
        offset: Option<usize>,
    ) -> Option<BoundingBox> {
        boundary::calculate_animation_bounds(self, animations, Tolerance::new(tolerance), to_frame, offset)
    }

    /// Searches an animation for the shortest interval after which its pose repeats.
    ///
    /// Frames are resolved in sequence and compared against every earlier frame
    /// until two match within the tolerance, detecting the true visual loop of
    /// animations whose declared keyframe range disagrees with their actual
    /// period.
    ///
    /// # Arguments
    /// * `animation` - The animation timeline to search.
    /// * `tolerance` - The maximum accumulated difference at which two frames are considered identical.
    /// * `minimum_frame` - The shortest interval to accept, which suppresses degenerate one-frame matches. Defaults to one.
    /// * `maximum_frame` - The highest frame to search before abandoning the search. Defaults to the highest frame a frame counter holds, leaving the callback to bound the search.
    /// * `progress_callback` - Invoked with each frame index as it is evaluated; returning `false` abandons the search.
    ///
    /// # Returns
    /// An `Option` containing the detected `Cycle`, or `None` if the animation
    /// did not repeat within the frame limit or the callback abandoned the
    /// search.
    pub fn calculate_cycle(
        &self,
        animation: &Animation,
        tolerance: f32,
        minimum_frame: Option<i32>,
        maximum_frame: Option<i32>,
        mut progress_callback: impl FnMut(usize) -> bool,
    ) -> Option<Cycle> {
        let minimum_loop_length = minimum_frame.unwrap_or(1);
        let frame_ceiling = maximum_frame.unwrap_or(i32::MAX);

        let mut history: Vec<Vec<FrameData>> = Vec::new();
        let mut frame = 0;

        while frame <= frame_ceiling {
            if !progress_callback(frame as usize) { return None; }

            let current = resolve_frame(self, Some(animation), frame, None);

            for (past_frame, past) in history.iter().enumerate() {
                if frame - (past_frame as i32) < minimum_loop_length { continue; }

                if periodicity::calculate_difference(&current, past) <= tolerance {
                    return Some(Cycle { start: past_frame as i32, end: frame });
                }
            }

            history.push(current);
            frame = frame.checked_add(1)?;
        }

        None
    }
}
