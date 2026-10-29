//! Public facade for the story chapters' treasures.
//!
//! Each story chapter awards one treasure per stage, graded Inferior, Normal or
//! Superior. The chapter's data file groups those treasures into sets, and a set
//! grants its effect once every treasure in it is held, scaled by the grades
//! held. The engine keeps ten chapter slots: three for Empire of Cats, one it
//! never fills, three for Into the Future and three for Cats of the Cosmos.

mod data;
mod effect;
mod store;
mod text;

pub use data::{TreasureData, TreasureDataError, TreasureGroup, GROUPS, GROUP_STAGES};
pub use effect::{enemy_money, unit_recharge, TraitBonus, TreasureEffect};
pub use store::{data_file, label_file, name_file, set_file, TreasureLevels, Treasures, ARC_SLOTS, SLOTS, STAGES};
pub use text::{TreasureText, TreasureTextError};
