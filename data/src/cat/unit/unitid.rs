//! Parsing of a Cat unit's per-form combat statistics.
//!
//! The engine stores these in a `unit<id>.csv` file alongside the unit's other
//! assets, with one row per evolutionary form and no header.

use crate::combat::{Column, Entity, EntityError, Faction, entity};
use crate::common::columns;
use crate::common::file::Separator;

/// Parses a per-unit Cat statistic file into one entity per evolutionary form.
///
/// The file carries no header, and each row describes one form in ascending
/// order beginning with the normal form. Units with fewer than four forms
/// declare correspondingly fewer rows.
///
/// # Arguments
/// * `bytes` - The raw, decrypted byte slice of a unit's `unit<id>.csv` file.
/// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
///
/// # Returns
/// A `Result` containing the parsed entities in form order on success, or an
/// `EntityError` if the file contained no rows wide enough to be interpreted
/// as combat data.
pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Entity>, EntityError> {
    entity::parse_rows(bytes.as_ref(), 0, separator, from_row)
}

/// The column mapping this parser applies, in the order it applies it.
///
/// Published so a consumer can read the layout of a `unit<id>.csv` row from
/// the parser's own table instead of restating it.
pub const COLUMNS: &[Column] = columns::columns! {
    hitpoints: 0;
    knockbacks: 1;
    speed: 2;
    attack_1_damage: 3;
    attack_cooldown: 4, Double;
    standing_range: 5;
    eoc1_cost: 6;
    cooldown: 7, Double;
    hitbox_position: 8;
    hitbox_width: 9;
    trait_red: 10;
    legacy_weak_against: 11;
    area_attack: 12;
    time_until_attack_1: 13;
    minimum_z_layer: 14;
    maximum_z_layer: 15;
    trait_floating: 16;
    trait_dark: 17;
    trait_metal: 18;
    trait_traitless: 19;
    trait_angel: 20;
    trait_alien: 21;
    trait_zombie: 22;
    strong_against: 23;
    knockback_chance: 24;
    freeze_chance: 25;
    freeze_duration: 26;
    slow_chance: 27;
    slow_duration: 28;
    resist: 29;
    massive_damage: 30;
    critical_chance: 31;
    attack_only: 32;
    double_bounty: 33;
    base_destroyer: 34;
    wave_chance: 35;
    wave_level: 36;
    weaken_chance: 37;
    weaken_duration: 38;
    weaken_to: 39;
    strengthen_threshold: 40;
    strengthen_boost: 41;
    survive: 42;
    is_metal: 43;
    long_distance_1_anchor: 44;
    long_distance_1_span: 45;
    wave_immune: 46;
    wave_block: 47;
    knockback_immune: 48;
    freeze_immune: 49;
    slow_immune: 50;
    weaken_immune: 51;
    zombie_killer: 52;
    witch_killer: 53;
    trait_witch: 54;
    attack_count_total: 55, Raw, -1;
    boss_wave_immune: 56;
    time_before_death: 57, Raw, -1;
    attack_count_state: 58;
    attack_2_damage: 59;
    attack_3_damage: 60;
    time_until_attack_2: 61;
    time_until_attack_3: 62;
    attack_1_abilities: 63;
    attack_2_abilities: 64;
    attack_3_abilities: 65;
    spawn_animation_type: 66, Raw, -1;
    soul_animation_type: 67, Raw, -1;
    spawn_animation_flag: 68;
    use_gudetama_soul: 69;
    barrier_breaker_chance: 70;
    warp_chance: 71;
    warp_duration: 72;
    warp_distance_anchor: 73, Quarter;
    warp_distance_span: 74, Quarter;
    warp_immune: 75;
    trait_eva: 76;
    eva_killer: 77;
    trait_relic: 78;
    curse_immune: 79;
    insanely_tough: 80;
    insane_damage: 81;
    savage_blow_chance: 82;
    savage_blow_boost: 83;
    dodge_chance: 84;
    dodge_duration: 85;
    surge_chance: 86;
    surge_spawn_anchor: 87, Quarter;
    surge_spawn_span: 88, Quarter;
    surge_level: 89;
    toxic_immune: 90;
    surge_immune: 91;
    curse_chance: 92;
    curse_duration: 93;
    mini_wave_flag: 94;
    shield_pierce_chance: 95;
    trait_aku: 96;
    colossus_slayer: 97;
    soulstrike: 98;
    long_distance_2_flag: 99;
    long_distance_2_anchor: 100;
    long_distance_2_span: 101;
    long_distance_3_flag: 102;
    long_distance_3_anchor: 103;
    long_distance_3_span: 104;
    behemoth_slayer: 105;
    behemoth_dodge_chance: 106;
    behemoth_dodge_duration: 107;
    mini_surge_flag: 108;
    counter_surge: 109;
    conjure_unit_id: 110, Raw, -1;
    sage_slayer: 111;
    metal_killer_percent: 112;
    explosion_chance: 113;
    explosion_spawn_anchor: 114, Quarter;
    explosion_spawn_span: 115, Quarter;
    explosion_immune: 116;
    drain_immune: 117;
};

fn from_row(cols: &[&str]) -> Entity {
    entity::build(cols, Faction::Cat, COLUMNS)
}
