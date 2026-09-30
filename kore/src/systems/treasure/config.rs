use std::collections::BTreeMap;

use nyanko::chapter::treasure::{TreasureLevels, ARC_SLOTS, SLOTS, STAGES};
use serde::{Deserialize, Serialize};

pub const ARCS: [&str; 3] = ["Empire of Cats", "Into the Future", "Cats of the Cosmos"];

const CHAPTER_PERCENT: u32 = 100;

pub const FULL_PERCENT: u32 = CHAPTER_PERCENT * ARC_SLOTS[0].len() as u32;

const SUPERIOR: i32 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub arcs: [String; 3],
    pub grades: BTreeMap<usize, i32>,
}

impl Config {
    pub fn arc(&self, arc: usize) -> u32 {
        self.arcs
            .get(arc)
            .and_then(|entry| entry.trim().trim_end_matches('%').trim().parse::<u32>().ok())
            .map_or(FULL_PERCENT, |percent| percent.min(FULL_PERCENT))
    }

    pub fn level(&self, slot: usize, stage: usize) -> i32 {
        self.grades.get(&key(slot, stage)).copied().unwrap_or_else(|| self.derived(slot, stage))
    }

    pub fn levels(&self) -> TreasureLevels {
        let mut levels = [[0; STAGES]; SLOTS];

        for (slot, row) in levels.iter_mut().enumerate() {
            for (stage, level) in row.iter_mut().enumerate() {
                *level = self.level(slot, stage);
            }
        }

        levels
    }

    pub fn set_arc(&mut self, arc: usize, entry: String) {
        let Some(held) = self.arcs.get_mut(arc) else {
            return;
        };

        *held = entry;

        if let Some(slots) = ARC_SLOTS.get(arc) {
            self.grades.retain(|key, _| !slots.contains(&(key / STAGES)));
        }
    }

    pub fn set_level(&mut self, slot: usize, stage: usize, level: i32) {
        if level == self.derived(slot, stage) {
            self.grades.remove(&key(slot, stage));
        } else {
            self.grades.insert(key(slot, stage), level);
        }
    }

    pub fn is_tuned(&self, arc: usize) -> bool {
        ARC_SLOTS.get(arc).is_some_and(|slots| self.grades.keys().any(|key| slots.contains(&(key / STAGES))))
    }

    fn derived(&self, slot: usize, stage: usize) -> i32 {
        let Some((arc, chapter)) = ARC_SLOTS
            .iter()
            .enumerate()
            .find_map(|(arc, slots)| slots.iter().position(|held| *held == slot).map(|chapter| (arc, chapter as u32)))
        else {
            return 0;
        };

        let fill = self.arc(arc).saturating_sub(chapter * CHAPTER_PERCENT).min(CHAPTER_PERCENT);
        let owned = (STAGES as u32 * fill).div_ceil(CHAPTER_PERCENT) as usize;

        if stage < owned { SUPERIOR } else { 0 }
    }
}

fn key(slot: usize, stage: usize) -> usize {
    slot * STAGES + stage
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each chapter is 100% of its arc, so an arc's percent fills chapter one
    // first, then two, then three.
    #[test]
    fn an_arc_percent_fills_its_chapters_in_order() {
        let mut config = Config::default();

        config.set_arc(0, "150".to_owned());
        config.set_arc(1, "0%".to_owned());

        let levels = config.levels();
        let owned = |slot: usize| levels[slot].iter().filter(|level| **level == SUPERIOR).count();

        assert_eq!([0, 1, 2].map(owned), [49, 25, 0]);
        assert_eq!([4, 5, 6].map(owned), [0, 0, 0]);
        assert_eq!([7, 8, 9].map(owned), [49, 49, 49], "an untouched arc is fully collected");
        // Slot 3 is the one the game never loads treasures into.
        assert_eq!(owned(3), 0);
    }

    #[test]
    fn an_arc_entry_overwrites_the_treasures_under_it() {
        let mut config = Config::default();

        config.set_level(4, 10, 1);
        config.set_level(7, 10, 2);
        assert_eq!(config.level(4, 10), 1);
        assert!(config.is_tuned(1));

        config.set_arc(1, "300".to_owned());

        assert_eq!(config.level(4, 10), SUPERIOR);
        assert!(!config.is_tuned(1));
        assert_eq!(config.level(7, 10), 2, "other arcs keep their own tuning");
    }

    #[test]
    fn an_entry_past_the_cap_is_pulled_back() {
        let mut config = Config::default();

        config.set_arc(2, "450%".to_owned());

        assert_eq!(config.arc(2), 300);
        assert_eq!(Config::default().arc(0), 300);
    }
}

