//! Parsing of the shared enemy combat statistic table.
//!
//! The engine stores every enemy's statistics in a single `t_unit.csv` table
//! carrying two header lines, with one row per enemy in identifier order.

use crate::combat::{Column, Entity, EntityError, Faction, entity};
use crate::common::columns;
use crate::common::file::Separator;

const HEADER_LINES: usize = 2;

/// Parses the shared enemy statistic table into one entity per declared enemy.
///
/// The table carries two leading header lines, which are skipped before the
/// remaining rows are read in file order. The position of an entity within the
/// returned vector is its internal enemy identifier.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of the `t_unit.csv` file.
/// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
///
/// # Returns
/// A `Result` containing the parsed entities indexed by enemy identifier on
/// success, or an `EntityError` if the file contained no rows wide enough to
/// be interpreted as combat data.
pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Entity>, EntityError> {
    entity::parse_rows(bytes.as_ref(), HEADER_LINES, separator, from_row)
}

/// Parses a single row of the shared enemy statistic table by enemy identifier.
///
/// This avoids materializing the entire table when only one enemy is required.
/// The two leading header lines are skipped before the identifier is applied as
/// a row offset.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of the `t_unit.csv` file.
/// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
/// * `id` - The internal enemy identifier, used as a zero-based row offset past the header.
/// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
///
/// # Returns
/// An `Option` containing the parsed entity, or `None` if the identifier lies
/// beyond the end of the table or addresses a row too narrow to be combat data.
pub fn parse_row<B: AsRef<[u8]>>(bytes: B, id: usize, separator: Option<Separator>) -> Option<Entity> {
    entity::parse_single(bytes.as_ref(), HEADER_LINES, id, separator, from_row)
}

/// The column mapping this parser applies, in the order it applies it.
///
/// Published so a consumer can read the layout of a `t_unit.csv` row from the
/// parser's own table instead of restating it. The Conjure identifier has no
/// column in this layout and is always the absent sentinel of negative one.
pub const COLUMNS: &[Column] = columns::columns! {
    hitpoints: 0;
    knockbacks: 1;
    speed: 2;
    attack_1_damage: 3;
    attack_cooldown: 4, Double;
    standing_range: 5;
    cash_drop: 6;
    hitbox_position: 7;
    hitbox_width: 8;
    legacy_strong_against: 9;
    trait_red: 10;
    area_attack: 11;
    time_until_attack_1: 12;
    trait_floating: 13;
    trait_dark: 14;
    trait_metal: 15;
    trait_traitless: 16;
    trait_angel: 17;
    trait_alien: 18;
    trait_zombie: 19;
    knockback_chance: 20;
    freeze_chance: 21;
    freeze_duration: 22;
    slow_chance: 23;
    slow_duration: 24;
    critical_chance: 25;
    base_destroyer: 26;
    wave_chance: 27;
    wave_level: 28;
    weaken_chance: 29;
    weaken_duration: 30;
    weaken_to: 31;
    strengthen_threshold: 32;
    strengthen_boost: 33;
    survive: 34;
    long_distance_1_anchor: 35;
    long_distance_1_span: 36;
    wave_immune: 37;
    wave_block: 38;
    knockback_immune: 39;
    freeze_immune: 40;
    slow_immune: 41;
    weaken_immune: 42;
    burrow_amount: 43;
    burrow_distance: 44, Quarter;
    revive_count: 45;
    revive_time: 46;
    revive_hp: 47;
    trait_witch: 48;
    trait_dojo: 49;
    attack_count_total: 50, Raw, -1;
    time_before_death: 51, Raw, -1;
    attack_count_state: 52;
    spawn_animation_type: 53, Raw, -1;
    soul_animation_type: 54, Raw, -1;
    attack_2_damage: 55;
    attack_3_damage: 56;
    time_until_attack_2: 57;
    time_until_attack_3: 58;
    attack_1_abilities: 59;
    attack_2_abilities: 60;
    attack_3_abilities: 61;
    spawn_animation_flag: 62;
    use_gudetama_soul: 63;
    barrier_hitpoints: 64;
    warp_chance: 65;
    warp_duration: 66;
    warp_distance_anchor: 67, Quarter;
    warp_distance_span: 68, Quarter;
    trait_starred_alien: 69;
    warp_immune: 70;
    trait_eva: 71;
    trait_relic: 72;
    curse_chance: 73;
    curse_duration: 74;
    savage_blow_chance: 75;
    savage_blow_boost: 76;
    dodge_chance: 77;
    dodge_duration: 78;
    toxic_chance: 79;
    toxic_damage: 80;
    surge_chance: 81;
    surge_spawn_anchor: 82, Quarter;
    surge_spawn_span: 83, Quarter;
    surge_level: 84;
    surge_immune: 85;
    mini_wave_flag: 86;
    shield_hitpoints: 87;
    shield_regen: 88;
    death_surge_chance: 89;
    death_surge_spawn_anchor: 90, Quarter;
    death_surge_spawn_span: 91, Quarter;
    death_surge_level: 92;
    trait_aku: 93;
    trait_colossus: 94;
    long_distance_2_flag: 95;
    long_distance_2_anchor: 96;
    long_distance_2_span: 97;
    long_distance_3_flag: 98;
    long_distance_3_anchor: 99;
    long_distance_3_span: 100;
    trait_behemoth: 101;
    mini_surge_flag: 102;
    counter_surge: 103;
    trait_sage: 104;
    curse_immune: 105;
    explosion_chance: 106;
    explosion_spawn_anchor: 107, Quarter;
    explosion_spawn_span: 108, Quarter;
    explosion_immune: 109;
    trait_kaijin: 110;
    drain_chance: 111;
    drain_percent: 112;
};

fn from_row(cols: &[&str]) -> Entity {
    let mut unit = entity::build(cols, Faction::Enemy, COLUMNS);
    unit.conjure_unit_id = -1;
    unit
}
