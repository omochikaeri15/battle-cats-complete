use nyanko::chapter::stage::{BattlegroundEntry, BossType, EnemyAmount};
use serde::{Deserialize, Serialize};

use crate::domains::enemy::filter::evaluation::{passes_at, target_magnification};
use crate::domains::enemy::filter::EnemyFilterState;
use crate::systems::combat::registry::Magnification;

use super::range::{CompiledStatRange, StatRange};
use super::StageLookupContext;

#[derive(Default, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct EnemyFilter {
    pub is_exclude: bool,
    pub name_or_id: String,
    pub amount: StatRange,
    pub start_frame: StatRange,
    pub respawn_min: StatRange,
    pub respawn_max: StatRange,
    pub base_hp_perc: StatRange,
    pub layer_min: StatRange,
    pub layer_max: StatRange,
    pub magnification: StatRange,
    pub atk_magnification: StatRange,
    pub score: StatRange,
    pub time_flag: StatRange,
    pub kill_count: StatRange,
    pub boss_type: Option<u32>,
    pub is_base: Option<bool>,
    #[serde(skip)]
    pub attributes: EnemyFilterState,
}

impl EnemyFilter {
    pub fn is_active(&self) -> bool {
        !self.name_or_id.trim().is_empty()
            || self.amount.is_active()
            || self.start_frame.is_active()
            || self.respawn_min.is_active()
            || self.respawn_max.is_active()
            || self.base_hp_perc.is_active()
            || self.layer_min.is_active()
            || self.layer_max.is_active()
            || self.magnification.is_active()
            || self.atk_magnification.is_active()
            || self.score.is_active()
            || self.time_flag.is_active()
            || self.kill_count.is_active()
            || self.boss_type.is_some()
            || self.is_base.is_some()
            || self.attributes.is_active()
    }

    pub(crate) fn compile(&self) -> CompiledEnemyFilter {
        let name_or_id = self.name_or_id.trim().to_lowercase();
        let parsed_id = name_or_id.parse::<u32>().ok();

        CompiledEnemyFilter {
            is_exclude: self.is_exclude,
            name_or_id,
            parsed_id,
            amount: self.amount.compile(0),
            start_frame: self.start_frame.compile(0),
            respawn_min: self.respawn_min.compile(0),
            respawn_max: self.respawn_max.compile(0),
            base_hp_perc: self.base_hp_perc.compile(0),
            layer_min: self.layer_min.compile(0),
            layer_max: self.layer_max.compile(0),
            magnification: self.magnification.compile(0),
            atk_magnification: self.atk_magnification.compile(0),
            score: self.score.compile(0),
            time_flag: self.time_flag.compile(0),
            kill_count: self.kill_count.compile(0),
            boss_type: self.boss_type,
            is_base: self.is_base,
            attributes: self.attributes.is_active().then(|| self.attributes.clone()),
        }
    }
}

pub(crate) struct CompiledEnemyFilter {
    pub is_exclude: bool,
    pub name_or_id: String,
    pub parsed_id: Option<u32>,
    pub amount: CompiledStatRange,
    pub start_frame: CompiledStatRange,
    pub respawn_min: CompiledStatRange,
    pub respawn_max: CompiledStatRange,
    pub base_hp_perc: CompiledStatRange,
    pub layer_min: CompiledStatRange,
    pub layer_max: CompiledStatRange,
    pub magnification: CompiledStatRange,
    pub atk_magnification: CompiledStatRange,
    pub score: CompiledStatRange,
    pub time_flag: CompiledStatRange,
    pub kill_count: CompiledStatRange,
    pub boss_type: Option<u32>,
    pub is_base: Option<bool>,
    pub attributes: Option<EnemyFilterState>,
}

impl CompiledEnemyFilter {
    pub(crate) fn matches(&self, enemy: &BattlegroundEntry, crown_mag: u32, ctx: &StageLookupContext) -> bool {
        let internal_amount = if let EnemyAmount::Limit(val) = enemy.amount { val as i64 } else { 0 };

        let internal_boss_type = match enemy.boss_type {
            BossType::None => 0,
            BossType::Boss => 1,
            BossType::ScreenShake => 2,
            BossType::Unknown(val) => val,
        };

        if self.boss_type.is_some_and(|expected| internal_boss_type != expected) { return false; }
        if self.is_base.is_some_and(|expected| enemy.is_base != expected) { return false; }
        if !self.amount.matches(internal_amount) { return false; }
        if !self.start_frame.matches(enemy.start_frame as i64) { return false; }
        if !self.respawn_min.matches(enemy.respawn_min as i64) { return false; }
        if !self.respawn_max.matches(enemy.respawn_max as i64) { return false; }
        if !self.base_hp_perc.matches(enemy.base_hp_perc as i64) { return false; }
        if !self.layer_min.matches(enemy.layer_min as i64) { return false; }
        if !self.layer_max.matches(enemy.layer_max as i64) { return false; }
        if !self.magnification.matches(enemy.magnification as i64) { return false; }
        if !self.atk_magnification.matches(enemy.atk_magnification as i64) { return false; }
        if !self.score.matches(enemy.score as i64) { return false; }
        if !self.time_flag.matches(enemy.time_flag as i64) { return false; }
        if !self.kill_count.matches(enemy.kill_count as i64) { return false; }

        self.named(enemy, ctx.enemy_name_registry) && self.attributed(enemy, crown_mag, ctx)
    }

    fn named(&self, enemy: &BattlegroundEntry, enemy_name_registry: &[String]) -> bool {
        if self.name_or_id.is_empty() { return true; }
        if self.parsed_id == Some(enemy.enemy_id) { return true; }
        if self.parsed_id.is_some() { return false; }

        enemy_name_registry
            .get(enemy.enemy_id as usize)
            .filter(|name| !name.is_empty())
            .map_or_else(
                || format!("{:03}-e", enemy.enemy_id).contains(&self.name_or_id),
                |name| name.to_lowercase().contains(&self.name_or_id)
            )
    }

    fn attributed(&self, enemy: &BattlegroundEntry, crown_mag: u32, ctx: &StageLookupContext) -> bool {
        let Some(attributes) = &self.attributes else { return true; };
        let Some(entry) = ctx.enemy_registry.get(&enemy.enemy_id) else { return false; };

        let staged = |magnification: u32| i32::try_from(u64::from(magnification) * u64::from(crown_mag) / 100).unwrap_or(i32::MAX);
        let magnification = target_magnification(attributes).unwrap_or_else(|| Magnification {
            hitpoints: staged(enemy.magnification),
            attack: staged(enemy.atk_magnification),
        });

        passes_at(entry, attributes, magnification, ctx.treasure)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nyanko::chapter::stage::ScatCpuSetting;
    use nyanko::combat::{Entity, Faction, Identity};

    use crate::domains::enemy::filter::RangeInput;
    use crate::domains::enemy::scanner::EnemyEntry;
    use crate::systems::treasure::Bonus;
    use crate::{ItemStore, Vfs};

    use super::*;

    const RED: u32 = 1;
    const FLOATING: u32 = 2;

    fn foe(id: u32, stats: Entity) -> (u32, EnemyEntry) {
        let entry = EnemyEntry { id, name: String::new(), description: Vec::new(), stats, icon_path: None, atk_anim_frames: 0 };

        (id, entry)
    }

    fn card(identity: Identity) -> EnemyFilter {
        let mut filter = EnemyFilter::default();
        filter.attributes.active_identities.insert(identity);

        filter
    }

    // One card is one enemy: its attributes and its spawn ranges have to hold on the same
    // battleground row, never on two different rows of the same stage.
    #[test]
    fn a_card_scopes_its_attributes_to_the_row_it_matches() {
        let registry: HashMap<u32, EnemyEntry> = [
            foe(RED, Entity { faction: Faction::Enemy, trait_red: 1, attack_1_damage: 1000, ..Entity::default() }),
            foe(FLOATING, Entity { faction: Faction::Enemy, trait_floating: 1, attack_1_damage: 1000, ..Entity::default() }),
        ].into();

        let (vfs, items, treasure) = (Vfs::with_priority(&[]), ItemStore::default(), Bonus::default());
        let (locks, cpu, drops, buys, cats) = (HashMap::new(), ScatCpuSetting::default(), HashMap::new(), HashMap::new(), HashMap::new());

        let ctx = StageLookupContext {
            enemy_registry: &registry,
            enemy_name_registry: &[],
            treasure: &treasure,
            lock_registry: &locks,
            cpu_setting: &cpu,
            items: &items,
            vfs: &vfs,
            drop_chara_registry: &drops,
            unit_buy_registry: &buys,
            cat_name_registry: &cats,
        };

        let late_red = BattlegroundEntry { enemy_id: RED, start_frame: 300, magnification: 100, atk_magnification: 100, ..Default::default() };
        let buffed_floating = BattlegroundEntry { enemy_id: FLOATING, magnification: 600, atk_magnification: 600, ..Default::default() };

        let mut reds_from_100 = card(Identity::TraitRed);
        reds_from_100.start_frame.min = "100".into();
        let reds_from_100 = reds_from_100.compile();

        assert!(reds_from_100.matches(&late_red, 100, &ctx));
        assert!(!reds_from_100.matches(&buffed_floating, 100, &ctx));

        // 1000 attack at the row's own 600% is 6000, so the stage magnification is what clears the bar.
        let mut strong_floating = card(Identity::TraitFloating);
        strong_floating.attributes.stat_ranges.insert("Attack", RangeInput { min: "5000".into(), max: String::new() });
        let strong_floating = strong_floating.compile();

        assert!(strong_floating.matches(&buffed_floating, 100, &ctx));
        assert!(!strong_floating.matches(&late_red, 100, &ctx));

        let mut strong_red = card(Identity::TraitRed);
        strong_red.attributes.stat_ranges.insert("Attack", RangeInput { min: "5000".into(), max: String::new() });
        let strong_red = strong_red.compile();

        assert!(!strong_red.matches(&late_red, 100, &ctx), "the red borrowed the floating row's attack");
        assert!(!strong_red.matches(&buffed_floating, 100, &ctx));
    }
}
