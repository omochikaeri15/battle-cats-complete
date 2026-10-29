use serde::{Deserialize, Serialize};

use super::{TreasureData, TreasureEffect, GROUPS, GROUP_STAGES};

pub const SLOTS: usize = 10;

pub const STAGES: usize = 49;

pub const ARC_SLOTS: [[usize; 3]; 3] = [[0, 1, 2], [4, 5, 6], [7, 8, 9]];

const EMPTY_SLOT: usize = 3;

const SUPERIOR: i32 = 3;

pub type TreasureLevels = [[i32; STAGES]; SLOTS];

pub fn data_file(slot: usize) -> Option<String> {
    match slot {
        0..=2 => Some("treasureData0.csv".to_owned()),
        4..=6 => Some("treasureData1.csv".to_owned()),
        7..=9 => Some(format!("treasureData2_{}.csv", slot - 7)),
        _ => None,
    }
}

pub fn name_file(slot: usize, lang: &str) -> Option<String> {
    match slot {
        0..=2 => Some(format!("Treasure1_0_{lang}.csv")),
        4..=6 => Some(format!("Treasure1_1_{lang}.csv")),
        7..=9 => Some(format!("Treasure1_2_{}_{lang}.csv", slot - 7)),
        _ => None,
    }
}

pub fn set_file(slot: usize, lang: &str) -> Option<String> {
    match slot {
        0..=2 => Some(format!("Treasure3_0_{lang}.csv")),
        4..=6 => Some(format!("Treasure3_1_{lang}.csv")),
        7..=9 => Some(format!("Treasure3_2_{}_{lang}.csv", slot - 7)),
        _ => None,
    }
}

pub fn label_file(lang: &str) -> String {
    format!("Treasure2_{lang}.csv")
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Treasures {
    pub slots: [Option<TreasureData>; SLOTS],
}

impl Treasures {
    pub fn progress(&self, levels: &TreasureLevels) -> [[i32; GROUPS]; SLOTS] {
        let mut progress = [[0; GROUPS]; SLOTS];

        for (slot, (data, row)) in self.slots.iter().zip(&mut progress).enumerate() {
            let Some(data) = data.as_ref().filter(|_| slot != EMPTY_SLOT) else { continue };

            for (group, cell) in data.groups.iter().zip(row.iter_mut()) {
                let mut sum = 0i32;
                let mut complete = true;

                for stage in group.members().take(GROUP_STAGES) {
                    let level = usize::try_from(stage).ok().and_then(|stage| levels[slot].get(stage)).copied().unwrap_or(0);

                    if level <= 0 {
                        complete = false;
                        break;
                    }

                    sum = sum.wrapping_add(level);
                }

                if complete && sum != 0 {
                    *cell = sum.wrapping_mul(100).checked_div(group.count.wrapping_mul(SUPERIOR)).unwrap_or(0);
                }
            }
        }

        progress
    }

    pub fn value(&self, levels: &TreasureLevels, effect: TreasureEffect, chapter: Option<usize>) -> i32 {
        let progress = self.progress(levels);

        self.sum(effect, chapter, |slot, group, percent| {
            let held = progress[slot][group];

            if percent == 0 { held } else { held.wrapping_mul(percent) / 100 }
        })
    }

    pub fn capped(&self, levels: &TreasureLevels, effect: TreasureEffect, percent: i32, chapter: Option<usize>) -> i32 {
        let progress = self.progress(levels);

        self.sum(effect, chapter, |slot, group, _| progress[slot][group].wrapping_mul(percent) / 100)
    }

    pub fn uncapped(&self, effect: TreasureEffect, chapter: Option<usize>) -> i32 {
        self.sum(effect, chapter, |_, _, percent| percent)
    }

    pub fn weakened(&self, stat: i64, levels: &TreasureLevels, effect: TreasureEffect, chapter: Option<usize>) -> i64 {
        let gap = i64::from(self.uncapped(effect, chapter).wrapping_sub(self.value(levels, effect, chapter)));

        stat.wrapping_add(gap.wrapping_mul(stat) / 100)
    }

    fn sum(&self, effect: TreasureEffect, chapter: Option<usize>, each: impl Fn(usize, usize, i32) -> i32) -> i32 {
        let mut total = 0i32;

        for (slot, data) in self.slots.iter().enumerate() {
            let Some(data) = data else { continue };

            for (group, set) in data.groups.iter().enumerate().take(GROUPS) {
                if set.effect != effect.id() || (set.is_chapter_only() && chapter != Some(slot)) {
                    continue;
                }

                total = total.wrapping_add(each(slot, group, set.percent));
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapter::treasure::TreasureGroup;

    fn set(stages: &[i32], percent: i32, effect: i32) -> TreasureGroup {
        TreasureGroup {
            count: stages.len() as i32,
            stages: stages.iter().copied().chain([-1]).collect(),
            percent,
            effect,
            ..TreasureGroup::default()
        }
    }

    fn arc(groups: Vec<TreasureGroup>) -> Treasures {
        let data = TreasureData { groups };
        let mut treasures = Treasures::default();

        for slot in ARC_SLOTS[0] {
            treasures.slots[slot] = Some(data.clone());
        }

        treasures
    }

    #[test]
    fn three_superior_chapters_give_the_familiar_two_and_a_half() {
        let treasures = arc(vec![set(&[0, 1], 50, 9)]);
        let mut levels = [[0; STAGES]; SLOTS];

        for slot in ARC_SLOTS[0] {
            levels[slot][..2].fill(3);
        }

        assert_eq!(treasures.value(&levels, TreasureEffect::CatHealth, None), 150);
    }

    #[test]
    fn a_set_missing_one_treasure_gives_nothing() {
        let treasures = arc(vec![set(&[0, 1], 50, 9)]);
        let mut levels = [[0; STAGES]; SLOTS];

        levels[0][0] = 3;
        assert_eq!(treasures.progress(&levels)[0][0], 0);

        // Inferior and Superior average to two thirds of a full set.
        levels[0][1] = 1;
        assert_eq!(treasures.progress(&levels)[0][0], 66);
    }

    #[test]
    fn an_alien_without_treasures_fights_at_its_full_strength() {
        let treasures = arc(vec![set(&[0], 100, 16), set(&[1], 100, 16)]);
        let none = [[0; STAGES]; SLOTS];

        assert_eq!(treasures.uncapped(TreasureEffect::AlienWeakening, None), 600);
        assert_eq!(treasures.weakened(100, &none, TreasureEffect::AlienWeakening, None), 700);
    }

    #[test]
    fn a_chapter_only_set_needs_its_own_chapter() {
        let mut mask = set(&[0], 1000, 22);
        mask.chapter_only = 1;
        let treasures = arc(vec![mask]);

        assert_eq!(treasures.uncapped(TreasureEffect::StarredAlienType2, None), 0);
        assert_eq!(treasures.uncapped(TreasureEffect::StarredAlienType2, Some(1)), 1000);
    }

    #[test]
    fn the_empty_slot_loads_nothing() {
        assert_eq!(data_file(3), None);
        assert_eq!(data_file(8).as_deref(), Some("treasureData2_1.csv"));
        assert_eq!(name_file(9, "en").as_deref(), Some("Treasure1_2_2_en.csv"));
    }
}
