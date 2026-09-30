//! Text handling, column tables, and regional conventions shared by every parser.
//!
//! The engine's data files vary in encoding, delimiter, and column count between
//! regions and versions, so every parser normalizes its input and declares its
//! layout through the items re-exported here.

pub(crate) mod columns;
pub(crate) mod file;
pub(crate) mod stream;
mod variant;

pub use columns::{Column, FromColumn, Scale, apply, parse_cell};
pub use file::{BreakHandling, Separator, lookup, scrub, strip_html_tags};
pub use stream::{Reader, cell_value, declared_rows};
pub use variant::{ParseRegionError, Region, RegionMetadata};
