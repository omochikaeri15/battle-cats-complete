mod catalog;
mod config;

use nyanko::chapter::treasure::{enemy_money, unit_recharge, TraitBonus, TreasureEffect, ARC_SLOTS, SLOTS};
use nyanko::chapter::Category;
use nyanko::combat::{Entity, Faction};

pub use catalog::Catalog;
pub use config::{Config, ARCS, FULL_PERCENT};

const MAX_TECH: i32 = 29;
const FRUIT_PERCENT: i32 = 100;

const FRUITS: [TreasureEffect; 7] = [
    TreasureEffect::Red,
    TreasureEffect::Floating,
    TreasureEffect::Dark,
    TreasureEffect::Angel,
    TreasureEffect::Metal,
    TreasureEffect::Zombie,
    TreasureEffect::Alien,
];

const WEAKENINGS: [TreasureEffect; 5] = [
    TreasureEffect::AlienWeakening,
    TreasureEffect::StarredAlienWeakening,
    TreasureEffect::StarredAlienType2,
    TreasureEffect::StarredAlienType3,
    TreasureEffect::StarredAlienType4,
];

const OUTSIDE_STORY: usize = SLOTS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bonus {
    pub(crate) cat_health: i32,
    pub(crate) cat_attack: i32,
    pub(crate) enemy_money: i32,
    pub(crate) cat_recharge: i32,
    fruits: [i32; FRUITS.len()],
    weakenings: [[i32; WEAKENINGS.len()]; SLOTS + 1],
}

impl Bonus {
    pub fn resolve(catalog: &Catalog, config: &Config) -> Self {
        let levels = config.levels();
        let treasures = &catalog.treasures;
        let value = |effect| treasures.value(&levels, effect, None);

        let mut weakenings = [[0; WEAKENINGS.len()]; SLOTS + 1];

        for (index, row) in weakenings.iter_mut().enumerate() {
            let chapter = (index != OUTSIDE_STORY).then_some(index);

            for (gap, effect) in row.iter_mut().zip(WEAKENINGS) {
                *gap = treasures.uncapped(effect, chapter) - treasures.value(&levels, effect, chapter);
            }
        }

        Self {
            cat_health: value(TreasureEffect::CatHealth),
            cat_attack: value(TreasureEffect::CatAttack),
            enemy_money: value(TreasureEffect::EnemyMoney),
            cat_recharge: value(TreasureEffect::CatRecharge),
            fruits: FRUITS.map(|effect| treasures.capped(&levels, effect, FRUIT_PERCENT, None)),
            weakenings,
        }
    }

    pub fn fruit(&self, effect: TreasureEffect) -> TraitBonus {
        TraitBonus(FRUITS.iter().position(|fruit| *fruit == effect).map_or(0, |index| self.fruits[index]))
    }

    pub fn weakening(&self, stats: &Entity, chapter: Option<usize>) -> i32 {
        let Some(effect) = TreasureEffect::for_enemy(stats.trait_alien != 0, stats.trait_starred_alien) else {
            return 0;
        };

        let row = chapter.filter(|chapter| *chapter < SLOTS).unwrap_or(OUTSIDE_STORY);

        WEAKENINGS.iter().position(|weakening| *weakening == effect).map_or(0, |index| self.weakenings[row][index])
    }

    pub fn fruit_span(&self, stats: &Entity) -> (TraitBonus, TraitBonus) {
        let fruit_flags = [stats.trait_red, stats.trait_floating, stats.trait_dark, stats.trait_angel, stats.trait_metal, stats.trait_zombie, stats.trait_alien];
        let bare_flags = [stats.trait_traitless, stats.trait_witch, stats.trait_eva, stats.trait_relic, stats.trait_aku];

        let values: Vec<i32> = FRUITS
            .iter()
            .zip(fruit_flags)
            .filter(|(_, flag)| *flag != 0)
            .map(|(effect, _)| self.fruit(*effect).0)
            .chain(bare_flags.into_iter().filter(|flag| *flag != 0).map(|_| 0))
            .collect();

        let low = values.iter().copied().min().unwrap_or(0);
        let high = values.iter().copied().max().unwrap_or(0);

        (TraitBonus(low), TraitBonus(high))
    }

    pub fn strength(&self, stats: &Entity, percent: i32) -> f32 {
        let gap = if stats.faction == Faction::Enemy { self.weakening(stats, None) } else { 0 };

        percent as f32 / 100.0 * (100 + gap) as f32 / 100.0
    }

    pub fn money(&self, drop: i32) -> i32 {
        enemy_money(drop, MAX_TECH, self.enemy_money)
    }

    pub fn recharge(&self, cooldown: i32) -> i32 {
        unit_recharge(cooldown, MAX_TECH, self.cat_recharge)
    }
}

pub fn chapter(category: &Category, map_id: u32) -> Option<usize> {
    let arc = match category {
        Category::EmpireOfCats => 0,
        Category::IntoTheFuture => 1,
        Category::CatsOfTheCosmos => 2,
        _ => return None,
    };
    let map = usize::try_from(map_id).ok()?;

    ARC_SLOTS[arc].contains(&map).then_some(map)
}
