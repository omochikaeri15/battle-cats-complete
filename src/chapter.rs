//! Public facade for chapter, map, and stage data.
//!
//! The engine organizes its level content in three tiers: a chapter groups maps,
//! and a map orders stages. This module re-exports the aggregate structure for
//! each tier along with the parsers behind them.

pub mod category;
pub mod stage;
pub mod map;
pub mod treasure;

pub use crate::common::Separator;
pub use category::Category;
pub use stage::Stage;
pub use map::Map;
