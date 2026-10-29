//! Animation and rendering support for unit graphics.
//!
//! This module turns the engine's raw animation assets into geometry any canvas
//! implementation can draw, without itself depending on a rendering backend.
//! Parsing lives in [`rig`], the game's own animation logic in `engine`, the
//! geometry a caller consumes in [`animate`], the helpers reading that logic
//! back out in [`tools`], and the battle background composed the way the
//! engine draws it in [`backdrop`].
//!
//! This module requires the `graphics` feature.

pub mod animate;
pub mod backdrop;
mod engine;
pub mod rig;
pub mod tools;
