use std::collections::HashMap;

use nyanko::cat::unit::Equipment;
use nyanko::combat::Entity;

use crate::domains::cat::game::talents::apply_talent_stats;
use crate::domains::cat::scanner::CatEntry;

pub const GRADES: [&str; 5] = ["D", "C", "B", "A", "S"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    pub content: i32,
    pub attribute: Option<i32>,
}

const STRONG: i32 = 2;
const MASSIVE: i32 = 3;
const RESIST: i32 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Allowance {
    strong: u16,
    massive: u16,
    resist: u16,
}

fn targets(stats: &Entity) -> u16 {
    [
        stats.trait_red,
        stats.trait_floating,
        stats.trait_dark,
        stats.trait_metal,
        stats.trait_angel,
        stats.trait_alien,
        stats.trait_zombie,
        stats.trait_relic,
        stats.trait_traitless,
        stats.trait_witch,
        stats.trait_eva,
        stats.trait_aku,
    ]
        .iter()
        .enumerate()
        .filter(|(_, held)| **held > 0)
        .fold(0, |mask, (bit, _)| mask | (1 << bit))
}

impl Allowance {
    pub fn of(entry: &CatEntry) -> Self {
        let mut allowance = Self::default();

        for stats in entry.stats.iter().flatten() {
            allowance.absorb(stats);
        }

        let talented = entry.talent_data.as_ref().zip(entry.stats.iter().flatten().last());

        if let Some((talents, stats)) = talented {
            let levels: HashMap<u8, u8> =
                talents.groups.iter().enumerate().map(|(group, talent)| (group as u8, talent.max_level.max(1))).collect();

            allowance.absorb(&apply_talent_stats(stats, talents, &levels));
        }

        allowance
    }

    fn absorb(&mut self, stats: &Entity) {
        let aimed = targets(stats);

        if stats.strong_against > 0 {
            self.strong |= aimed;
        }

        if stats.massive_damage > 0 {
            self.massive |= aimed;
        }

        if stats.resist > 0 {
            self.resist |= aimed;
        }
    }

    pub fn permits(&self, kind: Kind) -> bool {
        let mask = match kind.content {
            STRONG => self.strong,
            MASSIVE => self.massive,
            RESIST => self.resist,
            _ => return true,
        };

        kind.attribute.and_then(|bit| u32::try_from(bit).ok()).is_some_and(|bit| bit < 16 && mask & (1 << bit) != 0)
    }
}

pub fn kind(orbs: &[Equipment], orb: u32) -> Option<Kind> {
    let held = orbs.get(orb as usize)?;

    Some(Kind { content: held.content?, attribute: held.attribute })
}

pub fn grade(orbs: &[Equipment], orb: u32) -> Option<usize> {
    orbs.get(orb as usize).and_then(|held| held.grade_id).and_then(|grade| usize::try_from(grade).ok())
}

pub fn kinds(orbs: &[Equipment]) -> Vec<Kind> {
    let mut seen: Vec<Kind> = Vec::new();

    for held in orbs {
        let Some(content) = held.content else {
            continue;
        };
        let found = Kind { content, attribute: held.attribute };

        if !seen.contains(&found) {
            seen.push(found);
        }
    }

    seen
}

pub fn effects(kinds: &[Kind]) -> Vec<i32> {
    let mut seen: Vec<i32> = Vec::new();

    for kind in kinds {
        if !seen.contains(&kind.content) {
            seen.push(kind.content);
        }
    }

    seen
}

pub fn attributes(kinds: &[Kind], content: i32) -> Vec<i32> {
    kinds.iter().filter(|kind| kind.content == content).filter_map(|kind| kind.attribute).collect()
}

pub fn grades(orbs: &[Equipment], wanted: Kind) -> Vec<(usize, u32)> {
    orbs.iter()
        .enumerate()
        .filter(|(_, held)| held.content == Some(wanted.content) && held.attribute == wanted.attribute)
        .filter_map(|(orb, held)| Some((usize::try_from(held.grade_id?).ok()?, u32::try_from(orb).ok()?)))
        .collect()
}

pub fn pick(orbs: &[Equipment], wanted: Kind, grade: usize) -> Option<u32> {
    let offered = grades(orbs, wanted);

    offered
        .iter()
        .find(|(held, _)| *held == grade)
        .or_else(|| offered.iter().max_by_key(|(held, _)| *held))
        .map(|(_, orb)| *orb)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Strong vs Red earns the Strong orb for Red only; Attack and ability orbs are open to everyone.
    #[test]
    fn a_gated_orb_needs_the_ability_against_that_trait() {
        let mut allowance = Allowance::default();

        allowance.absorb(&Entity { strong_against: 1, trait_red: 1, ..Entity::default() });

        assert!(allowance.permits(Kind { content: STRONG, attribute: Some(0) }));
        assert!(!allowance.permits(Kind { content: STRONG, attribute: Some(5) }));
        assert!(!allowance.permits(Kind { content: MASSIVE, attribute: Some(0) }));
        assert!(allowance.permits(Kind { content: 0, attribute: Some(5) }));
        assert!(allowance.permits(Kind { content: 18, attribute: None }));
    }
}
