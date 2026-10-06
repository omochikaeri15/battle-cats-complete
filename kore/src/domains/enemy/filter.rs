pub mod evaluation;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use nyanko::combat::Identity;

use crate::systems::treasure::Bonus;

pub const ATTACK_TYPE_IDENTITIES: &[Identity] = &[
    Identity::SingleAttack,
    Identity::AreaAttack,
    Identity::OmniStrike,
    Identity::LongDistance,
    Identity::MultiHit,
];

#[derive(Clone, Copy, PartialEq, Default, Debug, Hash)]
pub enum MatchMode {
    #[default]
    And,
    Or,
}

#[derive(Clone, PartialEq, Default, Debug, Hash)]
pub struct RangeInput {
    pub min: String,
    pub max: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnemyFilterState {
    pub is_open: bool,
    pub active_identities: HashSet<Identity>,
    pub match_mode: MatchMode,
    pub adv_ranges: HashMap<Identity, HashMap<&'static str, RangeInput>>,
    pub mag_input: String,
    pub stat_ranges: HashMap<&'static str, RangeInput>,
    pub treasure: Bonus,
}

impl Default for EnemyFilterState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_identities: HashSet::new(),
            match_mode: MatchMode::And,
            adv_ranges: HashMap::new(),
            mag_input: String::new(),
            stat_ranges: HashMap::new(),
            treasure: Bonus::default(),
        }
    }
}

impl EnemyFilterState {
    pub fn is_active(&self) -> bool {
        !self.active_identities.is_empty()
            || self.stat_ranges.values().any(|r| !r.min.is_empty() || !r.max.is_empty())
    }
}

fn unordered<T: Hash>(items: impl Iterator<Item = T>) -> u64 {
    items
        .map(|item| {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            hasher.finish()
        })
        .fold(0, u64::wrapping_add)
}

impl Hash for EnemyFilterState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.match_mode.hash(state);
        self.mag_input.hash(state);
        unordered(self.stat_ranges.iter()).hash(state);
        unordered(self.active_identities.iter().map(|identity| {
            (identity, unordered(self.adv_ranges.get(identity).into_iter().flatten()))
        })).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(filter: &EnemyFilterState) -> u64 {
        let mut hasher = DefaultHasher::new();
        filter.hash(&mut hasher);

        hasher.finish()
    }

    // The stage list only re-filters when this hash moves, so every edit that changes what
    // matches has to move it, and opening the popup must not.
    #[test]
    fn the_hash_follows_what_the_filter_matches() {
        let blank = EnemyFilterState::default();

        let mut red = blank.clone();
        red.active_identities.insert(Identity::TraitRed);

        let mut ranged = red.clone();
        ranged.stat_ranges.insert("Attack", RangeInput { min: "5000".into(), max: String::new() });

        let mut tuned = ranged.clone();
        tuned.adv_ranges.entry(Identity::TraitRed).or_default().insert("Chance", RangeInput { min: "1".into(), max: String::new() });

        let digests = [digest(&blank), digest(&red), digest(&ranged), digest(&tuned)];

        for (index, one) in digests.iter().enumerate() {
            assert!(!digests[index + 1..].contains(one), "two different filters hashed alike");
        }

        let opened = EnemyFilterState { is_open: true, ..tuned.clone() };

        assert_eq!(digest(&opened), digest(&tuned));
    }
}
