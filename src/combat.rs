//! Shared combat statistics for both Cat and Enemy entities.
//!
//! Both factions describe their units with the same set of mechanical
//! properties, laid out differently in their respective source files. This
//! module holds the unified [`Entity`] both are parsed into, along with the
//! registry that recognizes which abilities a given entity carries.
//!
//! Each layout publishes its own column mapping as a slice of [`Column`], so a
//! consumer needing to know which raw index feeds which field can read the
//! parser's own table rather than restating it.

mod abilities;
pub(crate) mod entity;
mod glossary;

pub use abilities::{
    Ability, AttrUnit, AttrValue, Attribute, Identity, REGISTRY, Stored, TalentValue, get_ability,
    get_talent,
};
pub use crate::common::Scale;
pub use crate::common::Separator;
pub use entity::{Column, Entity, EntityError, Faction};
pub use glossary::{Glossary, GlossaryEntry, GlossaryError, TraitLabel, TraitLabelError, TraitLabels};
