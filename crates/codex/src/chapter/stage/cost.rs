use serde::{Deserialize, Serialize};

use crate::chapter::Category;
use crate::common::columns::FromColumn;

const CURRENCY_SCALE: u32 = 1000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum CostType {
    #[default]
    Energy,
    Item,
    Catamin,
}

impl CostType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CataminGrade {
    A,
    B,
    C,
}

impl CataminGrade {
    pub const fn from_key(key: u32) -> Self {
        match key {
            0 => Self::A,
            1 => Self::B,
            _ => Self::C,
        }
    }

    pub const fn key(self) -> u32 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResolvedCost {
    pub id: Option<u32>,
    pub value: u32,
}

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
