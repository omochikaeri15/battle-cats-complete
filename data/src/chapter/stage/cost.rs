//! Decoding of the packed entry cost a stage charges.
//!
//! A stage's cost column holds plain energy in most chapters. The Catamin
//! chapter, and any map whose metadata header sets the item flag, pack a
//! currency identifier above the amount instead. The two are selected by
//! unrelated means and are never both in effect.

use serde::{Deserialize, Serialize};

use crate::chapter::Category;
use crate::common::columns::FromColumn;

const CURRENCY_SCALE: u32 = 1000;

/// Selects how a stage charges its entry cost.
///
/// The metadata header's flag column encodes only [`CostType::Energy`] and
/// [`CostType::Item`]. The engine hardcodes the Catamin scheme to the Catamin
/// chapter and records it in no file, so [`CostType::Catamin`] comes from
/// [`CostType::of`] rather than from a parser.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum CostType {
    /// The stage charges the cost column as energy directly.
    #[default]
    Energy,
    /// The stage charges an item, whose identifier and quantity the column packs.
    Item,
    /// The stage charges Catamin, whose grade and quantity the column packs.
    Catamin,
}

impl CostType {
    /// Returns the scheme a stage actually charges under.
    ///
    /// A stage of the Catamin chapter charges Catamin whatever its map declares,
    /// since the engine hardcodes that and records it in no file. Every other
    /// chapter charges under the scheme its header declares.
    ///
    /// # Arguments
    /// * `category` - The chapter the stage belongs to.
    /// * `declared` - The scheme the map's metadata header declares.
    ///
    /// # Returns
    /// A `CostType` holding the scheme in effect for the stage.
    pub fn of(category: &Category, declared: Self) -> Self {
        match category {
            Category::CataminStages => Self::Catamin,
            _ => declared,
        }
    }
}

impl FromColumn for CostType {
    fn from_column(text: &str) -> Option<Self> {
        text.parse::<i32>().ok().map(|flag| match flag {
            1 => Self::Item,
            _ => Self::Energy,
        })
    }
}

/// The grade of Catamin a Catamin stage charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CataminGrade {
    /// Catamin A, selected by a packed key of zero.
    A,
    /// Catamin B, selected by a packed key of one.
    B,
    /// Catamin C, selected by a packed key of two or above.
    C,
}

impl CataminGrade {
    /// Returns the grade a packed key names.
    ///
    /// # Arguments
    /// * `key` - The upper portion of the packed cost column.
    ///
    /// # Returns
    /// A `CataminGrade` holding the grade the key names, saturating at
    /// [`CataminGrade::C`].
    pub const fn from_key(key: u32) -> Self {
        match key {
            0 => Self::A,
            1 => Self::B,
            _ => Self::C,
        }
    }

    /// Returns the packed key that names this grade.
    ///
    /// # Returns
    /// A `u32` holding the key, which is the identifier [`resolve_energy`]
    /// reports for a Catamin cost.
    pub const fn key(self) -> u32 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
        }
    }
}

/// A stage's entry cost, decoded into the currency it is charged in.
///
/// [`ResolvedCost::id`] is read in the space the [`CostType`] given to
/// [`resolve_energy`] names: an item identifier for [`CostType::Item`], a
/// [`CataminGrade::key`] for [`CostType::Catamin`], and nothing for
/// [`CostType::Energy`]. Catamin grades have no entry in the item tables, so a
/// caller drawing one maps the key itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResolvedCost {
    /// The currency charged, or `None` when the cost is plain energy.
    pub id: Option<u32>,
    /// The amount of that currency charged.
    pub value: u32,
}

/// Decodes a stage's cost column into the currency and amount it charges.
///
/// Under [`CostType::Energy`] the column is the amount itself. The other two
/// schemes pack a currency identifier above the amount. A Catamin key past two
/// saturates to [`CataminGrade::C`].
///
/// # Arguments
/// * `cost_type` - The scheme in effect for the stage, from [`CostType::of`].
/// * `energy` - The raw cost column of the stage's metadata row.
///
/// # Returns
/// A `ResolvedCost` holding the currency the stage charges and how much of it.
pub fn resolve_energy(cost_type: CostType, energy: u32) -> ResolvedCost {
    let packed_id = energy / CURRENCY_SCALE;
    let quantity = energy % CURRENCY_SCALE;

    match cost_type {
        CostType::Catamin => ResolvedCost {
            id: Some(CataminGrade::from_key(packed_id).key()),
            value: quantity,
        },
        CostType::Item => ResolvedCost {
            id: Some(packed_id),
            value: quantity
        },
        _ => ResolvedCost {
            id: None,
            value: energy
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(cost_type: CostType, energy: u32) -> (Option<u32>, u32) {
        let cost = resolve_energy(cost_type, energy);
        (cost.id, cost.value)
    }

    #[test]
    fn energy_resolves_to_itself_and_names_no_currency() {
        assert_eq!(resolved(CostType::Energy, 180), (None, 180));
        assert_eq!(resolved(CostType::Energy, 259030), (None, 259030));
    }

    #[test]
    fn an_item_cost_unpacks_its_identifier_and_quantity() {
        assert_eq!(resolved(CostType::Item, 259010), (Some(259), 10));
        assert_eq!(resolved(CostType::Item, 259100), (Some(259), 100));
    }

    #[test]
    fn an_item_cost_is_never_second_guessed() {
        assert_eq!(resolved(CostType::Item, 300), (Some(0), 300));
        assert_eq!(resolved(CostType::Item, 259000), (Some(259), 0));
        assert_eq!(resolved(CostType::Item, 0), (Some(0), 0));
    }

    #[test]
    fn catamin_keys_select_the_grade() {
        assert_eq!(resolved(CostType::Catamin, 30), (Some(0), 30));
        assert_eq!(resolved(CostType::Catamin, 1030), (Some(1), 30));
        assert_eq!(resolved(CostType::Catamin, 2030), (Some(2), 30));
    }

    #[test]
    fn a_catamin_key_past_the_last_grade_saturates() {
        assert_eq!(resolved(CostType::Catamin, 9030), (Some(2), 30));
    }

    #[test]
    fn a_catamin_quantity_is_never_guarded() {
        assert_eq!(resolved(CostType::Catamin, 0), (Some(0), 0));
        assert_eq!(resolved(CostType::Catamin, 2000), (Some(2), 0));
    }

    #[test]
    fn the_catamin_chapter_overrides_what_the_header_declares() {
        assert_eq!(CostType::of(&Category::CataminStages, CostType::Energy), CostType::Catamin);
        assert_eq!(CostType::of(&Category::CataminStages, CostType::Item), CostType::Catamin);
        assert_eq!(CostType::of(&Category::EventStages, CostType::Item), CostType::Item);
        assert_eq!(CostType::of(&Category::EmpireOfCats, CostType::Energy), CostType::Energy);
    }

    #[test]
    fn a_grade_survives_a_round_trip_through_its_key() {
        for grade in [CataminGrade::A, CataminGrade::B, CataminGrade::C] {
            assert_eq!(CataminGrade::from_key(grade.key()), grade);
        }
    }
}
