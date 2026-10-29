use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityError {
    EmptyFile,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid combat entities."),
        }
    }
}

impl error::Error for EntityError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    #[default]
    Cat,
    Enemy,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub faction: Faction,
    pub hitpoints: i32,
    pub knockbacks: i32,
    pub speed: i32,
    pub attack_1_damage: i32,
    pub attack_2_damage: i32,
    pub attack_3_damage: i32,
    pub attack_cooldown: i32,
    pub standing_range: i32,
    pub hitbox_position: i32,
    pub hitbox_width: i32,
    pub legacy_weak_against: i32,
    pub legacy_strong_against: i32,
    pub eoc1_cost: i32,
    pub cooldown: i32,
    pub cash_drop: i32,
    pub minimum_z_layer: i32,
    pub maximum_z_layer: i32,
    pub trait_red: i32,
    pub trait_floating: i32,
    pub trait_dark: i32,
    pub trait_metal: i32,
    pub trait_traitless: i32,
    pub trait_angel: i32,
    pub trait_alien: i32,
    pub trait_zombie: i32,
    pub trait_witch: i32,
    pub trait_eva: i32,
    pub trait_relic: i32,
    pub trait_aku: i32,
    pub trait_dojo: i32,
    pub trait_starred_alien: i32,
    pub trait_behemoth: i32,
    pub trait_colossus: i32,
    pub trait_sage: i32,
    pub trait_kaijin: i32,
    pub area_attack: i32,
    pub time_until_attack_1: i32,
    pub time_until_attack_2: i32,
    pub time_until_attack_3: i32,
    pub attack_1_abilities: i32,
    pub attack_2_abilities: i32,
    pub attack_3_abilities: i32,
    pub attack_count_total: i32,
    pub attack_count_state: i32,
    pub time_before_death: i32,
    pub spawn_animation_type: i32,
    pub soul_animation_type: i32,
    pub spawn_animation_flag: i32,
    pub use_gudetama_soul: i32,
    pub attack_only: i32,
    pub strong_against: i32,
    pub massive_damage: i32,
    pub insane_damage: i32,
    pub resist: i32,
    pub insanely_tough: i32,
    pub is_metal: i32,
    pub double_bounty: i32,
    pub zombie_killer: i32,
    pub soulstrike: i32,
    pub colossus_slayer: i32,
    pub sage_slayer: i32,
    pub behemoth_slayer: i32,
    pub behemoth_dodge_chance: i32,
    pub behemoth_dodge_duration: i32,
    pub witch_killer: i32,
    pub eva_killer: i32,
    pub metal_killer_percent: i32,
    pub barrier_breaker_chance: i32,
    pub shield_pierce_chance: i32,
    pub conjure_unit_id: i32,
    pub knockback_chance: i32,
    pub freeze_chance: i32,
    pub freeze_duration: i32,
    pub slow_chance: i32,
    pub slow_duration: i32,
    pub weaken_chance: i32,
    pub weaken_duration: i32,
    pub weaken_to: i32,
    pub curse_chance: i32,
    pub curse_duration: i32,
    pub warp_chance: i32,
    pub warp_duration: i32,
    pub warp_distance_anchor: i32,
    pub warp_distance_span: i32,
    pub dodge_chance: i32,
    pub dodge_duration: i32,
    pub savage_blow_chance: i32,
    pub savage_blow_boost: i32,
    pub critical_chance: i32,
    pub strengthen_threshold: i32,
    pub strengthen_boost: i32,
    pub survive: i32,
    pub base_destroyer: i32,
    pub wave_chance: i32,
    pub wave_level: i32,
    pub mini_wave_flag: i32,
    pub wave_block: i32,
    pub surge_chance: i32,
    pub surge_spawn_anchor: i32,
    pub surge_spawn_span: i32,
    pub surge_level: i32,
    pub mini_surge_flag: i32,
    pub counter_surge: i32,
    pub explosion_chance: i32,
    pub explosion_spawn_anchor: i32,
    pub explosion_spawn_span: i32,
    pub death_surge_chance: i32,
    pub death_surge_spawn_anchor: i32,
    pub death_surge_spawn_span: i32,
    pub death_surge_level: i32,
    pub long_distance_1_anchor: i32,
    pub long_distance_1_span: i32,
    pub long_distance_2_flag: i32,
    pub long_distance_2_anchor: i32,
    pub long_distance_2_span: i32,
    pub long_distance_3_flag: i32,
    pub long_distance_3_anchor: i32,
    pub long_distance_3_span: i32,
    pub wave_immune: i32,
    pub knockback_immune: i32,
    pub freeze_immune: i32,
    pub slow_immune: i32,
    pub weaken_immune: i32,
    pub warp_immune: i32,
    pub curse_immune: i32,
    pub toxic_immune: i32,
    pub surge_immune: i32,
    pub explosion_immune: i32,
    pub boss_wave_immune: i32,
    pub barrier_hitpoints: i32,
    pub shield_hitpoints: i32,
    pub shield_regen: i32,
    pub burrow_amount: i32,
    pub burrow_distance: i32,
    pub revive_count: i32,
    pub revive_time: i32,
    pub revive_hp: i32,
    pub toxic_chance: i32,
    pub toxic_damage: i32,
    pub drain_chance: i32,
    pub drain_percent: i32,
    pub drain_immune: i32,
    pub recharge_cut: i32,
    pub has_unknown_abilities: i32,
}

impl Entity {
    pub fn attack_cycle(&self, frames: i32) -> i32 {
        let mut effective_foreswing = self.time_until_attack_1;

        if self.attack_3_damage > 0 && self.time_until_attack_3 > 0 {
            effective_foreswing = self.time_until_attack_3;
        }
        else if self.attack_2_damage > 0 && self.time_until_attack_2 > 0 {
            effective_foreswing = self.time_until_attack_2;
        }

        let cooldown_frames = self.attack_cooldown.saturating_sub(1);

        (effective_foreswing + cooldown_frames).max(frames)
    }

    pub fn legacy_matchup(&self, target: &Self) -> bool {
        self.legacy_strong_against != 0 && target.legacy_weak_against != 0 && self.faction != target.faction
    }
}

pub type Column = columns::Column<Entity>;

pub(crate) fn build(cols: &[&str], faction: Faction, table: &[Column]) -> Entity {
    let mut unit = Entity { faction, ..Entity::default() };
    let past_table = columns::apply(cols, table, &mut unit);

    unit.has_unknown_abilities = trailing_unknowns(cols, past_table);
    unit
}

fn trailing_unknowns(cols: &[&str], start: usize) -> i32 {
    i32::from(cols.iter().skip(start).any(|col| {
        let value = col.trim().parse::<i32>().unwrap_or(0);
        value != 0 && value != -1
    }))
}

fn split_row(line: &str, separator: char) -> Vec<&str> {
    let body = line.split_once("//").map_or(line, |(head, _)| head).trim_end();
    let mut cols: Vec<&str> = body.split(separator).collect();

    while cols.last().is_some_and(|col| col.trim().is_empty()) {
        cols.pop();
    }

    cols
}

pub(crate) fn parse_rows(bytes: &[u8], skip: usize, separator: Option<Separator>, from_row: fn(&[&str]) -> Entity) -> Result<Vec<Entity>, EntityError> {
    let content = file::scrub(bytes);
    let separator = file::resolve(separator, &content);
    let entities: Vec<Entity> = content
        .lines()
        .skip(skip)
        .filter_map(|line| {
            let cols = split_row(line, separator);
            (cols.len() >= 10).then(|| from_row(&cols))
        })
        .collect();
    if entities.is_empty() {
        return Err(EntityError::EmptyFile);
    }
    Ok(entities)
}

pub(crate) fn parse_single(bytes: &[u8], skip: usize, id: usize, separator: Option<Separator>, from_row: fn(&[&str]) -> Entity) -> Option<Entity> {
    let content = file::scrub(bytes);
    let separator = file::resolve(separator, &content);
    let target_line = content.lines().skip(skip).nth(id)?;
    let cols = split_row(target_line, separator);
    if cols.len() < 10 {
        return None;
    }
    Some(from_row(&cols))
}

#[cfg(test)]
mod tests {
    use crate::{cat::unitid, enemy::t_unit};

    use super::*;

    const NARROW_ROW: &str = "100,1,10,50,20,300,75,60,0,120";

    #[test]
    fn split_row_drops_the_comment_and_trailing_blanks() {
        assert_eq!(split_row("1,2,3, // ねこ占い師", ','), ["1", "2", "3"]);
        assert_eq!(split_row("1,2,3\t//ネコ杏子", ','), ["1", "2", "3"]);
        assert_eq!(split_row("1,2,3 // ちびタンクネコ", ','), ["1", "2", "3"]);
        assert_eq!(split_row("1,,3,,", ','), ["1", "", "3"]);
    }

    #[test]
    fn comments_never_reach_the_columns() {
        let expected = unitid::parse(NARROW_ROW, None).unwrap();

        for shape in [
            format!("{NARROW_ROW}, // ねこ占い師"),
            format!("{NARROW_ROW}\t//ネコ杏子"),
            format!("{NARROW_ROW} // ちびタンクネコ"),
        ] {
            assert_eq!(unitid::parse(&shape, None).unwrap(), expected);
        }
    }

    #[test]
    fn a_value_glued_to_a_comment_survives() {
        let forms = unitid::parse(format!("{NARROW_ROW},5 // my note"), None).unwrap();
        assert_eq!(forms[0].trait_red, 5);
    }

    #[test]
    fn column_tables_cover_every_index_once() {
        for table in [unitid::COLUMNS, t_unit::COLUMNS] {
            let mut indices: Vec<usize> = table.iter().map(|column| column.index).collect();
            indices.sort_unstable();
            indices.dedup();
            assert_eq!(indices.len(), table.len());
            assert_eq!(indices.last().copied(), Some(table.len() - 1));

            let mut fields: Vec<&str> = table.iter().map(|column| column.field).collect();
            fields.sort_unstable();
            fields.dedup();
            assert_eq!(fields.len(), table.len());
        }
    }

    #[test]
    fn every_column_reaches_the_field_it_names() {
        for (table, faction) in [(unitid::COLUMNS, Faction::Cat), (t_unit::COLUMNS, Faction::Enemy)] {
            for column in table {
                let mut cols = vec!["0"; table.len()];
                cols[column.index] = "8";

                let serialized = serde_json::to_value(build(&cols, faction, table)).unwrap();
                let stored = serialized.get(column.field).and_then(serde_json::Value::as_i64);

                assert_eq!(stored, Some(i64::from(column.scale.apply(8))), "{}", column.field);
            }
        }
    }
}
