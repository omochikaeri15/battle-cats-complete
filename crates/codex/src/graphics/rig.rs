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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RigError {
    EmptyFile,
    TruncatedHeader,
    NoPartHeader,
    ImageDecodeFailed,
    NoSpriteCuts,
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

#[derive(Debug, Clone)]
pub struct Rig {
    pub model: Model,
    pub sheet: SpriteSheet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cycle {
    pub start: i32,
    pub end: i32,
}

impl Cycle {
    pub fn length(&self) -> i32 {
        self.end - self.start
    }
}

impl Rig {
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

    pub fn calculate_bounds(
        &self,
        animations: &[&Animation],
        tolerance: f32,
        to_frame: Option<i32>,
        offset: Option<usize>,
    ) -> Option<BoundingBox> {
        boundary::calculate_animation_bounds(self, animations, Tolerance::new(tolerance), to_frame, offset)
    }

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
