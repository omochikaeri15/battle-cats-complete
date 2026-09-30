use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};

/// Represents errors that can occur while parsing raw combat statistic rows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityError {
    /// The supplied bytes yielded no rows wide enough to be treated as combat data.
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

/// Identifies which side of a battle an entity belongs to.
///
/// The two factions read their statistics from different source files with
/// different column orderings, so this flag records which layout produced the
/// surrounding [`Entity`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    /// A player-controlled Cat unit, parsed from a per-unit `unit<id>.csv` file.
    #[default]
    Cat,
    /// An opposing enemy unit, parsed from a row of the shared `t_unit.csv` table.
    Enemy,
}

/// The complete mechanical combat definition of a single entity form.
///
/// The two factions store their statistics in separate files whose columns
/// diverge in order and meaning; both normalize into this layout, with the origin
/// recorded in [`Entity::faction`]. Fields absent from a faction's source file
/// keep their default.
///
/// Durations are frames at thirty per second, chances are whole percentages, and
/// distances are already divided by four where the raw column quadruples them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// The side of the battle this entity fights on, and therefore which source layout produced it.
    pub faction: Faction,
    /// The total health pool before any level or treasure scaling is applied.
    pub hitpoints: i32,
    /// The number of times the entity is repelled before it is defeated.
    pub knockbacks: i32,
    /// The horizontal movement rate in engine distance units per frame.
    pub speed: i32,
    /// The damage dealt by the first attack in the entity's attack sequence.
    pub attack_1_damage: i32,
    /// The damage dealt by the second attack, or zero when the entity has a single-hit attack.
    pub attack_2_damage: i32,
    /// The damage dealt by the third attack, or zero when the entity has fewer than three hits.
    pub attack_3_damage: i32,
    /// The recovery period in frames between the end of one attack sequence and the start of the next.
    pub attack_cooldown: i32,
    /// The distance at which the entity halts and begins attacking.
    pub standing_range: i32,
    /// The offset of the damage hitbox relative to the entity's own position.
    pub hitbox_position: i32,
    /// The width of the damage hitbox.
    pub hitbox_width: i32,
    /// Non-zero when the entity is a valid target of the legacy strong-against matchup.
    pub legacy_weak_against: i32,
    /// Non-zero when the entity applies the legacy strong-against matchup to opponents that accept it.
    pub legacy_strong_against: i32,
    /// The deployment cost in currency during the first chapter, before chapter cost multipliers.
    pub eoc1_cost: i32,
    /// The redeployment delay in frames after the entity is summoned.
    pub cooldown: i32,
    /// The currency awarded to the opposing player when this entity is defeated.
    pub cash_drop: i32,
    /// The lower bound of the random draw layer used to order overlapping sprites.
    pub minimum_z_layer: i32,
    /// The upper bound of the random draw layer used to order overlapping sprites.
    pub maximum_z_layer: i32,
    /// Non-zero when the entity carries the Red trait.
    pub trait_red: i32,
    /// Non-zero when the entity carries the Floating trait.
    pub trait_floating: i32,
    /// Non-zero when the entity carries the Black trait.
    pub trait_dark: i32,
    /// Non-zero when the entity carries the Metal trait.
    pub trait_metal: i32,
    /// Non-zero when the entity carries the Traitless classification.
    pub trait_traitless: i32,
    /// Non-zero when the entity carries the Angel trait.
    pub trait_angel: i32,
    /// Non-zero when the entity carries the Alien trait.
    pub trait_alien: i32,
    /// Non-zero when the entity carries the Zombie trait.
    pub trait_zombie: i32,
    /// Non-zero when the entity carries the Witch trait.
    pub trait_witch: i32,
    /// Non-zero when the entity carries the Eva Angel trait.
    pub trait_eva: i32,
    /// Non-zero when the entity carries the Relic trait.
    pub trait_relic: i32,
    /// Non-zero when the entity carries the Aku trait.
    pub trait_aku: i32,
    /// Non-zero when the entity carries the Dojo classification used by base-defense stages.
    pub trait_dojo: i32,
    /// Non-zero when the entity carries the Starred Alien trait.
    pub trait_starred_alien: i32,
    /// Non-zero when the entity carries the Behemoth trait.
    pub trait_behemoth: i32,
    /// Non-zero when the entity carries the Colossus trait.
    pub trait_colossus: i32,
    /// Non-zero when the entity carries the Sage trait.
    pub trait_sage: i32,
    /// Non-zero when the entity carries the Kaijin trait.
    pub trait_kaijin: i32,
    /// Non-zero when the entity damages every target within its hitbox rather than only the first.
    pub area_attack: i32,
    /// The frame within the attack animation at which the first attack applies its damage.
    pub time_until_attack_1: i32,
    /// The frame within the attack animation at which the second attack applies its damage.
    pub time_until_attack_2: i32,
    /// The frame within the attack animation at which the third attack applies its damage.
    pub time_until_attack_3: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the first attack.
    pub attack_1_abilities: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the second attack.
    pub attack_2_abilities: i32,
    /// A bitfield selecting which of the entity's abilities are carried by the third attack.
    pub attack_3_abilities: i32,
    /// The number of attacks that make up one complete attack sequence.
    pub attack_count_total: i32,
    /// The internal counter tracking which attack of the sequence is next to resolve.
    pub attack_count_state: i32,
    /// The delay in frames between the entity's death and the removal of its corpse.
    pub time_before_death: i32,
    /// The identifier of the animation played when the entity enters the battlefield.
    pub spawn_animation_type: i32,
    /// The identifier of the animation played when the entity is defeated.
    pub soul_animation_type: i32,
    /// Non-zero when the spawn animation is enabled for this entity.
    pub spawn_animation_flag: i32,
    /// Non-zero when the death animation is enabled for this entity.
    pub use_gudetama_soul: i32,
    /// Non-zero when the entity may only attack and cannot block advancing opponents.
    pub attack_only: i32,
    /// The damage multiplier applied against the traits this entity is strong against.
    pub strong_against: i32,
    /// The damage multiplier applied by the Massive Damage ability.
    pub massive_damage: i32,
    /// The damage multiplier applied by the Insane Damage ability.
    pub insane_damage: i32,
    /// The incoming damage reduction applied by the Resistant ability.
    pub resist: i32,
    /// The incoming damage reduction applied by the Insanely Tough ability.
    pub insanely_tough: i32,
    /// Non-zero when the entity behaves as a Metal target, taking one damage from ordinary hits.
    pub is_metal: i32,
    /// Non-zero when the entity yields twice the ordinary currency on defeat.
    pub double_bounty: i32,
    /// The damage multiplier applied against Zombie targets, which also prevents their revival.
    pub zombie_killer: i32,
    /// Non-zero when the entity's attacks pierce the burrowed state of Zombie targets.
    pub soulstrike: i32,
    /// The damage multiplier applied against Colossus targets.
    pub colossus_slayer: i32,
    /// The damage multiplier applied against Sage targets.
    pub sage_slayer: i32,
    /// The damage multiplier applied against Behemoth targets.
    pub behemoth_slayer: i32,
    /// The percentage chance of evading an incoming attack from a Behemoth target.
    pub behemoth_dodge_chance: i32,
    /// The duration in frames of the evasion granted against Behemoth targets.
    pub behemoth_dodge_duration: i32,
    /// The damage multiplier applied against Witch targets.
    pub witch_killer: i32,
    /// The damage multiplier applied against Eva Angel targets.
    pub eva_killer: i32,
    /// The percentage of the target's health dealt as damage against Metal targets.
    pub metal_killer_percent: i32,
    /// The percentage chance of destroying a target's barrier outright.
    pub barrier_breaker_chance: i32,
    /// The percentage chance of ignoring a target's shield.
    pub shield_pierce_chance: i32,
    /// The identifier of the unit summoned by the Conjure ability, or negative one when absent.
    pub conjure_unit_id: i32,
    /// The percentage chance of repelling the target on hit.
    pub knockback_chance: i32,
    /// The percentage chance of freezing the target on hit.
    pub freeze_chance: i32,
    /// The duration in frames of the freeze effect.
    pub freeze_duration: i32,
    /// The percentage chance of slowing the target on hit.
    pub slow_chance: i32,
    /// The duration in frames of the slow effect.
    pub slow_duration: i32,
    /// The percentage chance of weakening the target on hit.
    pub weaken_chance: i32,
    /// The duration in frames of the weaken effect.
    pub weaken_duration: i32,
    /// The percentage of its original attack power a weakened target retains.
    pub weaken_to: i32,
    /// The percentage chance of cursing the target, suppressing its trait-based abilities.
    pub curse_chance: i32,
    /// The duration in frames of the curse effect.
    pub curse_duration: i32,
    /// The percentage chance of warping the target backwards on hit.
    pub warp_chance: i32,
    /// The duration in frames the target remains immobilized after being warped.
    pub warp_duration: i32,
    /// The shortest distance a warped target is displaced.
    pub warp_distance_anchor: i32,
    /// The longest distance a warped target is displaced.
    pub warp_distance_span: i32,
    /// The percentage chance of evading an incoming attack.
    pub dodge_chance: i32,
    /// The duration in frames of the evasion effect.
    pub dodge_duration: i32,
    /// The percentage chance of triggering a Savage Blow.
    pub savage_blow_chance: i32,
    /// The additional damage percentage applied by a Savage Blow.
    pub savage_blow_boost: i32,
    /// The percentage chance of dealing a critical hit, which also damages Metal targets normally.
    pub critical_chance: i32,
    /// The remaining health percentage below which the Strengthen ability activates.
    pub strengthen_threshold: i32,
    /// The additional attack percentage granted once Strengthen activates.
    pub strengthen_boost: i32,
    /// The percentage chance of surviving a lethal hit with one health point remaining.
    pub survive: i32,
    /// The damage multiplier applied against enemy and player bases.
    pub base_destroyer: i32,
    /// The percentage chance of emitting a shockwave on attack.
    pub wave_chance: i32,
    /// The level of the emitted shockwave, which determines how far it travels.
    pub wave_level: i32,
    /// Non-zero when the emitted shockwave deals reduced mini-wave damage.
    pub mini_wave_flag: i32,
    /// Non-zero when the entity is immune to shockwaves and prevents them from passing.
    pub wave_block: i32,
    /// The percentage chance of creating a surge at a distant point on attack.
    pub surge_chance: i32,
    /// The nearest distance at which a surge may be created.
    pub surge_spawn_anchor: i32,
    /// The width of the range within which a surge may be created.
    pub surge_spawn_span: i32,
    /// The level of the created surge, which determines its duration.
    pub surge_level: i32,
    /// Non-zero when the created surge deals reduced mini-surge damage.
    pub mini_surge_flag: i32,
    /// Non-zero when the entity retaliates against surges that strike it.
    pub counter_surge: i32,
    /// The percentage chance of creating an explosion at a distant point on attack.
    pub explosion_chance: i32,
    /// The nearest distance at which an explosion may be created.
    pub explosion_spawn_anchor: i32,
    /// The width of the range within which an explosion may be created.
    pub explosion_spawn_span: i32,
    /// The percentage chance of creating a surge at the point of the entity's death.
    pub death_surge_chance: i32,
    /// The nearest distance at which a death surge may be created.
    pub death_surge_spawn_anchor: i32,
    /// The width of the range within which a death surge may be created.
    pub death_surge_spawn_span: i32,
    /// The level of the created death surge, which determines its duration.
    pub death_surge_level: i32,
    /// The nearest distance covered by the entity's first long-distance attack band.
    pub long_distance_1_anchor: i32,
    /// The width of the entity's first long-distance attack band.
    pub long_distance_1_span: i32,
    /// Non-zero when the second long-distance attack band is active.
    pub long_distance_2_flag: i32,
    /// The nearest distance covered by the entity's second long-distance attack band.
    pub long_distance_2_anchor: i32,
    /// The width of the entity's second long-distance attack band.
    pub long_distance_2_span: i32,
    /// Non-zero when the third long-distance attack band is active.
    pub long_distance_3_flag: i32,
    /// The nearest distance covered by the entity's third long-distance attack band.
    pub long_distance_3_anchor: i32,
    /// The width of the entity's third long-distance attack band.
    pub long_distance_3_span: i32,
    /// Non-zero when the entity ignores incoming shockwaves.
    pub wave_immune: i32,
    /// Non-zero when the entity ignores incoming knockback effects.
    pub knockback_immune: i32,
    /// Non-zero when the entity ignores incoming freeze effects.
    pub freeze_immune: i32,
    /// Non-zero when the entity ignores incoming slow effects.
    pub slow_immune: i32,
    /// Non-zero when the entity ignores incoming weaken effects.
    pub weaken_immune: i32,
    /// Non-zero when the entity ignores incoming warp effects.
    pub warp_immune: i32,
    /// Non-zero when the entity ignores incoming curse effects.
    pub curse_immune: i32,
    /// Non-zero when the entity ignores incoming toxic damage.
    pub toxic_immune: i32,
    /// Non-zero when the entity ignores incoming surges.
    pub surge_immune: i32,
    /// Non-zero when the entity ignores incoming explosions.
    pub explosion_immune: i32,
    /// Non-zero when the entity ignores the knockback inflicted by boss waves.
    pub boss_wave_immune: i32,
    /// The health pool of the barrier that must be broken before the entity itself takes damage.
    pub barrier_hitpoints: i32,
    /// The health pool of the regenerating shield that absorbs incoming damage.
    pub shield_hitpoints: i32,
    /// The percentage of shield health restored once the shield is broken.
    pub shield_regen: i32,
    /// The number of times the entity may burrow beneath attacks.
    pub burrow_amount: i32,
    /// The distance from the player base at which the entity begins to burrow.
    pub burrow_distance: i32,
    /// The number of times the entity returns to life after being defeated.
    pub revive_count: i32,
    /// The delay in frames between the entity's death and its revival.
    pub revive_time: i32,
    /// The percentage of maximum health the entity is restored to on revival.
    pub revive_hp: i32,
    /// The percentage chance of inflicting proportional toxic damage on hit.
    pub toxic_chance: i32,
    /// The percentage of the target's maximum health dealt as toxic damage.
    pub toxic_damage: i32,
    /// The percentage chance of restoring health proportional to the damage dealt.
    pub drain_chance: i32,
    /// The percentage of damage dealt that is restored as health.
    pub drain_percent: i32,
    /// Non-zero when the entity ignores incoming drain effects.
    pub drain_immune: i32,
    /// Non-zero when the source row carried trailing values this parser does not interpret.
    pub has_unknown_abilities: i32,
}

impl Entity {
    /// Calculates the total length in frames of one complete attack cycle.
    ///
    /// The cycle spans from the start of the attack animation through the frame
    /// on which the final hit lands and then through the recovery period. When
    /// the entity has multiple attacks, the effective wind-up is taken from the
    /// last attack that both deals damage and declares a strike frame.
    ///
    /// # Arguments
    /// * `frames` - The measured length of the attack animation, used as a lower bound when the statistical values imply a shorter cycle.
    ///
    /// # Returns
    /// An `i32` containing the greater of the computed cycle length and the supplied animation length.
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

    /// Reports whether the legacy strong-against matchup resolves against a target.
    ///
    /// The matchup predates the trait system, so neither side names a trait the
    /// other must carry. It applies when the attacker declares
    /// [`Entity::legacy_strong_against`], the target declares
    /// [`Entity::legacy_weak_against`], and the two fight on opposing sides.
    ///
    /// # Arguments
    /// * `target` - The entity receiving the attack.
    ///
    /// # Returns
    /// A `bool` that is true when both halves of the matchup are declared across opposing factions.
    pub fn legacy_matchup(&self, target: &Self) -> bool {
        self.legacy_strong_against != 0 && target.legacy_weak_against != 0 && self.faction != target.faction
    }
}

/// The column mapping of one of the raw statistic layouts.
///
/// The tables are [`crate::cat::unitid::COLUMNS`] and
/// [`crate::enemy::t_unit::COLUMNS`]. Columns are listed in the order the parser
/// applies them, and the highest [`columns::Column::index`] in a table is the
/// last column that layout understands. Anything beyond it is what
/// [`Entity::has_unknown_abilities`] reports on.
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
