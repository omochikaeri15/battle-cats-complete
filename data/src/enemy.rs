//! Public facade for Enemy unit data.
//!
//! This module re-exports the aggregate [`Unit`] structure, the shared table
//! bundle used to build it, and every specialized parser and error type behind
//! it, so callers never reach into the internal submodule layout.

pub mod unit;

pub use crate::common::Separator;
pub use unit::t_unit;
pub use unit::{EnemyName, EnemyNameError, EnemyPictureBook, EnemyPictureBookError, Tables, Unit};
