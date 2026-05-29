use crate::cat::unit::{Battle, TalentGroup};
use crate::common::data::img015;

/// Represents the mathematical unit of a raw attribute in the game engine.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AttrUnit {
    None,       // For Counts, Levels, raw hitpoints
    Percent,    // For Chances, Boosts, Reductions
    Frames,     // For Time and Durations
    Range,      // For Distances
}

/// Strict domain identifiers for Abilities.
/// Exported so the `core` shell can exhaustively match UI logic without ID collisions.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum AbilityIdentity {
    SingleAttack, AreaAttack, TargetRed, TargetFloat, TargetDark, TargetMetal, TargetAngel,
    TargetAlien, TargetZombie, TargetRelic, TargetAku, TargetTraitless, TargetWitch, TargetEva,
    AttackOnly, StrongAgainst, MassiveDamage, InsaneDamage, Resist, InsanelyTough, Metal,
    BaseDestroyer, DoubleBounty, ZombieKiller, Soulstrike, ColossusSlayer, SageSlayer,
    BehemothSlayer, WitchKiller, EvaKiller, WaveBlock, CounterSurge, Kamikaze, Stop, MultiHit,
    LongDistance, OmniStrike, Conjure, MetalKiller, WaveAttack, MiniWave, SurgeAttack,
    MiniSurge, Explosion, SavageBlow, CriticalHit, Strengthen, Survive, BarrierBreaker,
    ShieldPiercer, Dodge, Weaken, Freeze, Slow, Knockback, Curse, Warp, Unknown,
    ImmuneWave, ImmuneSurge, ImmuneExplosion, ImmuneWeaken, ImmuneFreeze, ImmuneSlow,
    ImmuneKnockback, ImmuneCurse, ImmuneToxic, ImmuneWarp, ImmuneBossWave, ResistWeaken,
    ResistFreeze, ResistSlow, ResistKnockback, ResistWave, ResistWarp, ResistCurse,
    ResistToxic, ResistSurge, CostDown, RecoverSpeedUp, MoveSpeedUp, AttackBuff,
    HealthBuff, TbaDown, ImproveKnockbacks
}

/// The pure domain definition of a Cat Ability.
pub struct Ability {
    pub identity: AbilityIdentity,
    pub talent_id: u8,
    pub icon_id: Option<usize>,
    pub name: &'static str,
    pub description: &'static str,
    pub schema: &'static [(&'static str, AttrUnit)],
    pub attributes: fn(&Battle) -> Vec<(&'static str, i32, AttrUnit)>,
    pub apply_talent: Option<fn(&mut Battle, val1: i32, val2: i32, group: &TalentGroup)>,
}

// --- DOMAIN HELPER FUNCTIONS ---

fn get_dur_val(v1: i32, v2: i32) -> i32 {
    if v1 != 0 { v1 } else { v2 }
}

pub fn get_talent(id: u8) -> Option<&'static Ability> {
    CAT_ABILITY_REGISTRY.iter().find(|ability| ability.talent_id == id)
}

// --- ENGINE ABILITY REGISTRY ---

pub static CAT_ABILITY_REGISTRY: &[Ability] = &[
    // --- SPECIAL HIDDEN ---
    Ability {
        identity: AbilityIdentity::SingleAttack,
        talent_id: 0,
        icon_id: Some(img015::ICON_SINGLE_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.area_attack == 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::AreaAttack,
        talent_id: 0,
        icon_id: Some(img015::ICON_AREA_ATTACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.area_attack == 1 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },

    // --- TRAITS ---
    Ability {
        identity: AbilityIdentity::TargetRed,
        talent_id: 33,
        icon_id: Some(img015::ICON_TRAIT_RED),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_red > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_red = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetFloat,
        talent_id: 34,
        icon_id: Some(img015::ICON_TRAIT_FLOATING),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_floating > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_floating = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetDark,
        talent_id: 35,
        icon_id: Some(img015::ICON_TRAIT_BLACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_dark > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_dark = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetMetal,
        talent_id: 36,
        icon_id: Some(img015::ICON_TRAIT_METAL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_metal > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_metal = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetAngel,
        talent_id: 37,
        icon_id: Some(img015::ICON_TRAIT_ANGEL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_angel > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_angel = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetAlien,
        talent_id: 38,
        icon_id: Some(img015::ICON_TRAIT_ALIEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_alien > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_alien = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetZombie,
        talent_id: 39,
        icon_id: Some(img015::ICON_TRAIT_ZOMBIE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_zombie > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_zombie = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetRelic,
        talent_id: 40,
        icon_id: Some(img015::ICON_TRAIT_RELIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_relic > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_relic = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetAku,
        talent_id: 57,
        icon_id: Some(img015::ICON_TRAIT_AKU),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_aku > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_aku = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetTraitless,
        talent_id: 41,
        icon_id: Some(img015::ICON_TRAIT_TRAITLESS),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_traitless > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_traitless = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetWitch,
        talent_id: 0,
        icon_id: Some(img015::ICON_WITCH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_witch > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_witch = 1),
    },
    Ability {
        identity: AbilityIdentity::TargetEva,
        talent_id: 0,
        icon_id: Some(img015::ICON_EVA),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.target_eva > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.target_eva = 1),
    },

    // --- DAMAGE / DEFENSE MODIFIERS ---
    Ability {
        identity: AbilityIdentity::AttackOnly,
        talent_id: 4,
        icon_id: Some(img015::ICON_ATTACK_ONLY),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.attack_only > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.attack_only = 1),
    },
    Ability {
        identity: AbilityIdentity::StrongAgainst,
        talent_id: 5,
        icon_id: Some(img015::ICON_STRONG_AGAINST),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.strong_against > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.strong_against = 1),
    },
    Ability {
        identity: AbilityIdentity::MassiveDamage,
        talent_id: 7,
        icon_id: Some(img015::ICON_MASSIVE_DAMAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.massive_damage > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.massive_damage = 1),
    },
    Ability {
        identity: AbilityIdentity::InsaneDamage,
        talent_id: 7,
        icon_id: Some(img015::ICON_INSANE_DAMAGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.insane_damage > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::Resist,
        talent_id: 6,
        icon_id: Some(img015::ICON_RESIST),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.resist > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.resist = 1),
    },
    Ability {
        identity: AbilityIdentity::InsanelyTough,
        talent_id: 6,
        icon_id: Some(img015::ICON_INSANELY_TOUGH),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.insanely_tough > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::Metal,
        talent_id: 43,
        icon_id: Some(img015::ICON_METAL),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.metal > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.metal = 1),
    },
    Ability {
        identity: AbilityIdentity::BaseDestroyer,
        talent_id: 12,
        icon_id: Some(img015::ICON_BASE_DESTROYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.base_destroyer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.base_destroyer = 1),
    },
    Ability {
        identity: AbilityIdentity::DoubleBounty,
        talent_id: 16,
        icon_id: Some(img015::ICON_DOUBLE_BOUNTY),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.double_bounty > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.double_bounty = 1),
    },
    Ability {
        identity: AbilityIdentity::ZombieKiller,
        talent_id: 14,
        icon_id: Some(img015::ICON_ZOMBIE_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.zombie_killer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.zombie_killer = 1),
    },
    Ability {
        identity: AbilityIdentity::Soulstrike,
        talent_id: 59,
        icon_id: Some(img015::ICON_SOULSTRIKE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.soulstrike == 2 || (stats.soulstrike > 0 && stats.zombie_killer > 0) { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.soulstrike = 2),
    },
    Ability {
        identity: AbilityIdentity::ColossusSlayer,
        talent_id: 63,
        icon_id: Some(img015::ICON_COLOSSUS_SLAYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.colossus_slayer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.colossus_slayer = 1),
    },
    Ability {
        identity: AbilityIdentity::SageSlayer,
        talent_id: 66,
        icon_id: Some(img015::ICON_SAGE_SLAYER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.sage_slayer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.sage_slayer = 1),
    },
    Ability {
        identity: AbilityIdentity::BehemothSlayer,
        talent_id: 64,
        icon_id: Some(img015::ICON_BEHEMOTH_SLAYER),
        name: "",
        description: "",
        schema: &[
            ("Dodge Chance", AttrUnit::Percent),
            ("Dodge Duration", AttrUnit::Frames)
        ],
        attributes: |stats| {
            if stats.behemoth_slayer > 0 {
                if stats.behemoth_dodge_chance > 0 {
                    vec![
                        ("Active", 1, AttrUnit::None),
                        ("Dodge Chance", stats.behemoth_dodge_chance, AttrUnit::Percent),
                        ("Dodge Duration", stats.behemoth_dodge_duration, AttrUnit::Frames),
                    ]
                } else {
                    vec![("Active", 1, AttrUnit::None)]
                }
            } else {
                vec![]
            }
        },
        apply_talent: Some(|stats, chance, duration, _| {
            stats.behemoth_slayer = 1;
            stats.behemoth_dodge_chance = if chance > 0 { chance } else { 5 };
            stats.behemoth_dodge_duration = if duration > 0 { duration } else { 30 };
        }),
    },
    Ability {
        identity: AbilityIdentity::WitchKiller,
        talent_id: 0,
        icon_id: Some(img015::ICON_WITCH_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.witch_killer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.witch_killer = 1),
    },
    Ability {
        identity: AbilityIdentity::EvaKiller,
        talent_id: 0,
        icon_id: Some(img015::ICON_EVA_KILLER),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.eva_killer > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.eva_killer = 1),
    },
    Ability {
        identity: AbilityIdentity::WaveBlock,
        talent_id: 0,
        icon_id: Some(img015::ICON_WAVE_BLOCK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.wave_block > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats, _, _, _| stats.wave_block = 1),
    },
    Ability {
        identity: AbilityIdentity::CounterSurge,
        talent_id: 68,
        icon_id: Some(img015::ICON_COUNTER_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.counter_surge > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.counter_surge = 1),
    },
    Ability {
        identity: AbilityIdentity::Kamikaze,
        talent_id: 0,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 2 {
                vec![("Attacks", stats.attack_count_total, AttrUnit::None)]
            } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::Stop,
        talent_id: 0,
        icon_id: None,
        name: "",
        description: "",
        schema: &[("Attacks", AttrUnit::None)],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 0 {
                vec![("Attacks", stats.attack_count_total, AttrUnit::None)]
            } else { vec![] }
        },
        apply_talent: None,
    },

    // --- ATTACK MECHANICS ---
    Ability {
        identity: AbilityIdentity::MultiHit,
        talent_id: 0,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.attack_2 > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::LongDistance,
        talent_id: 0,
        icon_id: Some(img015::ICON_LONG_DISTANCE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            let has_omni = (stats.long_distance_1_span < 0 || (stats.long_distance_1_span == 0 && stats.long_distance_1_anchor != 0)) ||
                (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
                (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)));

            let has_ld = (stats.long_distance_1_span > 0) ||
                (stats.long_distance_2_flag > 0 && stats.long_distance_2_span > 0) ||
                (stats.long_distance_3_flag > 0 && stats.long_distance_3_span > 0);

            if has_ld && !has_omni { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::OmniStrike,
        talent_id: 0,
        icon_id: Some(img015::ICON_OMNI_STRIKE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            let has_omni = (stats.long_distance_1_span < 0 || (stats.long_distance_1_span == 0 && stats.long_distance_1_anchor != 0)) ||
                (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
                (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)));

            if has_omni { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::Conjure,
        talent_id: 0,
        icon_id: Some(img015::ICON_CONJURE),
        name: "",
        description: "",
        schema: &[("Spirit ID", AttrUnit::None)],
        attributes: |stats| {
            if stats.conjure_unit_id > -1 { vec![("Spirit ID", stats.conjure_unit_id, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::MetalKiller,
        talent_id: 0,
        icon_id: Some(img015::ICON_METAL_KILLER),
        name: "",
        description: "",
        schema: &[("Damage", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.metal_killer_percent > 0 { vec![("Damage", stats.metal_killer_percent, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, percent, _, _| stats.metal_killer_percent = percent),
    },
    Ability {
        identity: AbilityIdentity::WaveAttack,
        talent_id: 17,
        icon_id: Some(img015::ICON_WAVE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
        ],
        attributes: |stats| {
            if stats.mini_wave_flag == 0 && stats.wave_chance > 0 {
                let maximum_reach = (332.5 + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32;
                vec![
                    ("Chance", stats.wave_chance, AttrUnit::Percent),
                    ("Level", stats.wave_level, AttrUnit::None),
                    ("Max Reach", maximum_reach, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: AbilityIdentity::MiniWave,
        talent_id: 62,
        icon_id: Some(img015::ICON_MINI_WAVE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
        ],
        attributes: |stats| {
            if stats.mini_wave_flag > 0 && stats.wave_chance > 0 {
                let maximum_reach = (332.5 + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32;
                vec![
                    ("Chance", stats.wave_chance, AttrUnit::Percent),
                    ("Level", stats.wave_level, AttrUnit::None),
                    ("Max Reach", maximum_reach, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.mini_wave_flag = 1; stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: AbilityIdentity::SurgeAttack,
        talent_id: 56,
        icon_id: Some(img015::ICON_SURGE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag == 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", stats.surge_chance, AttrUnit::Percent),
                    ("Level", stats.surge_level, AttrUnit::None),
                    ("Min Range", stats.surge_spawn_anchor, AttrUnit::Range),
                    ("Max Range", stats.surge_spawn_anchor + stats.surge_spawn_span, AttrUnit::Range),
                    ("Width", stats.surge_spawn_span, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.surge_chance += chance; stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: AbilityIdentity::MiniSurge,
        talent_id: 65,
        icon_id: Some(img015::ICON_MINI_SURGE),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag > 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", stats.surge_chance, AttrUnit::Percent),
                    ("Level", stats.surge_level, AttrUnit::None),
                    ("Min Range", stats.surge_spawn_anchor, AttrUnit::Range),
                    ("Max Range", stats.surge_spawn_anchor + stats.surge_spawn_span, AttrUnit::Range),
                    ("Width", stats.surge_spawn_span, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.mini_surge_flag = 1; stats.surge_chance += chance; stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: AbilityIdentity::Explosion,
        talent_id: 67,
        icon_id: Some(img015::ICON_EXPLOSION),
        name: "",
        description: "",
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
        ],
        attributes: |stats| {
            if stats.explosion_chance > 0 {
                vec![
                    ("Chance", stats.explosion_chance, AttrUnit::Percent),
                    ("Min Range", stats.explosion_spawn_anchor, AttrUnit::Range),
                    ("Max Range", stats.explosion_spawn_anchor + stats.explosion_spawn_span, AttrUnit::Range),
                    ("Width", stats.explosion_spawn_span, AttrUnit::Range),
                ]
            } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, group_data| {
            stats.explosion_chance += chance;
            stats.explosion_spawn_anchor = group_data.min_2 as i32 / 4;
            stats.explosion_spawn_span = group_data.min_3 as i32 / 4;
        }),
    },
    Ability {
        identity: AbilityIdentity::SavageBlow,
        talent_id: 50,
        icon_id: Some(img015::ICON_SAVAGE_BLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.savage_blow_chance > 0 { vec![("Chance", stats.savage_blow_chance, AttrUnit::Percent), ("Boost", stats.savage_blow_boost, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, boost, _| { stats.savage_blow_chance += chance; if boost > 0 { stats.savage_blow_boost = boost; } }),
    },
    Ability {
        identity: AbilityIdentity::CriticalHit,
        talent_id: 13,
        icon_id: Some(img015::ICON_CRITICAL_HIT),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.critical_chance > 0 { vec![("Chance", stats.critical_chance, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, _| stats.critical_chance += chance),
    },

    // --- STATE MODIFIERS ---
    Ability {
        identity: AbilityIdentity::Strengthen,
        talent_id: 10,
        icon_id: Some(img015::ICON_STRENGTHEN),
        name: "",
        description: "",
        schema: &[("HP", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.strengthen_threshold > 0 { vec![("HP", stats.strengthen_threshold, AttrUnit::Percent), ("Boost", stats.strengthen_boost, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, threshold, boost, _| {
            if stats.strengthen_boost == 0 {
                stats.strengthen_threshold = 100 - threshold;
                stats.strengthen_boost = boost;
            } else {
                stats.strengthen_boost += if threshold != 0 { threshold } else { boost };
            }
        }),
    },
    Ability {
        identity: AbilityIdentity::Survive,
        talent_id: 11,
        icon_id: Some(img015::ICON_SURVIVE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.survive > 0 { vec![("Chance", stats.survive, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, _| stats.survive += chance),
    },
    Ability {
        identity: AbilityIdentity::BarrierBreaker,
        talent_id: 15,
        icon_id: Some(img015::ICON_BARRIER_BREAKER),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.barrier_breaker_chance > 0 { vec![("Chance", stats.barrier_breaker_chance, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, _| stats.barrier_breaker_chance += chance),
    },
    Ability {
        identity: AbilityIdentity::ShieldPiercer,
        talent_id: 58,
        icon_id: Some(img015::ICON_SHIELD_PIERCER),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.shield_pierce_chance > 0 { vec![("Chance", stats.shield_pierce_chance, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, _| stats.shield_pierce_chance += chance),
    },
    Ability {
        identity: AbilityIdentity::Dodge,
        talent_id: 51,
        icon_id: Some(img015::ICON_DODGE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.dodge_chance > 0 { vec![("Chance", stats.dodge_chance, AttrUnit::Percent), ("Duration", stats.dodge_duration, AttrUnit::Frames)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, duration, _| { stats.dodge_chance += chance; stats.dodge_duration += duration; }),
    },
    Ability {
        identity: AbilityIdentity::Weaken,
        talent_id: 1,
        icon_id: Some(img015::ICON_WEAKEN),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Reduced To", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.weaken_chance > 0 { vec![("Chance", stats.weaken_chance, AttrUnit::Percent), ("Reduced To", stats.weaken_to, AttrUnit::Percent), ("Duration", stats.weaken_duration, AttrUnit::Frames)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.weaken_chance == 0 {
                stats.weaken_chance = chance; stats.weaken_duration = duration; stats.weaken_to = (100 - group_data.min_3) as i32;
            } else if group_data.text_id == 42 { stats.weaken_duration += get_dur_val(chance, duration); }
            else { stats.weaken_chance += chance; stats.weaken_duration += duration; }
        }),
    },
    Ability {
        identity: AbilityIdentity::Freeze,
        talent_id: 2,
        icon_id: Some(img015::ICON_FREEZE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.freeze_chance > 0 { vec![("Chance", stats.freeze_chance, AttrUnit::Percent), ("Duration", stats.freeze_duration, AttrUnit::Frames)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.freeze_chance == 0 { stats.freeze_chance = chance; stats.freeze_duration = duration; }
            else if group_data.text_id == 74 { stats.freeze_chance += chance; }
            else { stats.freeze_duration += get_dur_val(chance, duration); }
        }),
    },
    Ability {
        identity: AbilityIdentity::Slow,
        talent_id: 3,
        icon_id: Some(img015::ICON_SLOW),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.slow_chance > 0 { vec![("Chance", stats.slow_chance, AttrUnit::Percent), ("Duration", stats.slow_duration, AttrUnit::Frames)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.slow_chance == 0 { stats.slow_chance = chance; stats.slow_duration = duration; }
            else if group_data.text_id == 63 { stats.slow_chance += chance; }
            else { stats.slow_duration += get_dur_val(chance, duration); }
        }),
    },
    Ability {
        identity: AbilityIdentity::Knockback,
        talent_id: 8,
        icon_id: Some(img015::ICON_KNOCKBACK),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.knockback_chance > 0 { vec![("Chance", stats.knockback_chance, AttrUnit::Percent)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, _, _| stats.knockback_chance += chance),
    },
    Ability {
        identity: AbilityIdentity::Curse,
        talent_id: 60,
        icon_id: Some(img015::ICON_CURSE),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.curse_chance > 0 { vec![("Chance", stats.curse_chance, AttrUnit::Percent), ("Duration", stats.curse_duration, AttrUnit::Frames)] } else { vec![] }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.curse_chance == 0 { stats.curse_chance = chance; stats.curse_duration = duration; }
            else if group_data.text_id == 93 { stats.curse_duration += get_dur_val(chance, duration); }
            else { stats.curse_chance += chance; }
        }),
    },
    Ability {
        identity: AbilityIdentity::Warp,
        talent_id: 9,
        icon_id: Some(img015::ICON_WARP),
        name: "",
        description: "",
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames), ("Min Distance", AttrUnit::Range), ("Max Distance", AttrUnit::Range)],
        attributes: |stats| {
            if stats.warp_chance > 0 {
                vec![("Chance", stats.warp_chance, AttrUnit::Percent), ("Duration", stats.warp_duration, AttrUnit::Frames), ("Min Distance", stats.warp_distance_minimum, AttrUnit::Range), ("Max Distance", stats.warp_distance_maximum, AttrUnit::Range)]
            } else { vec![] }
        },
        apply_talent: None,
    },
    Ability {
        identity: AbilityIdentity::Unknown,
        talent_id: 0,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.has_unknown_abilities > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: None,
    },

    // --- IMMUNITIES ---
    Ability {
        identity: AbilityIdentity::ImmuneWave,
        talent_id: 48,
        icon_id: Some(img015::ICON_IMMUNE_WAVE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.wave_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.wave_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneSurge,
        talent_id: 55,
        icon_id: Some(img015::ICON_IMMUNE_SURGE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.surge_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.surge_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneExplosion,
        talent_id: 69,
        icon_id: Some(img015::ICON_IMMUNE_EXPLOSION),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.explosion_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.explosion_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneWeaken,
        talent_id: 44,
        icon_id: Some(img015::ICON_IMMUNE_WEAKEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.weaken_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.weaken_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneFreeze,
        talent_id: 45,
        icon_id: Some(img015::ICON_IMMUNE_FREEZE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.freeze_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.freeze_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneSlow,
        talent_id: 46,
        icon_id: Some(img015::ICON_IMMUNE_SLOW),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.slow_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.slow_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneKnockback,
        talent_id: 47,
        icon_id: Some(img015::ICON_IMMUNE_KNOCKBACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.knockback_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.knockback_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneCurse,
        talent_id: 29,
        icon_id: Some(img015::ICON_IMMUNE_CURSE),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.curse_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.curse_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneToxic,
        talent_id: 53,
        icon_id: Some(img015::ICON_IMMUNE_TOXIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.toxic_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.toxic_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneWarp,
        talent_id: 49,
        icon_id: Some(img015::ICON_IMMUNE_WARP),
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.warp_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.warp_immune = 1),
    },
    Ability {
        identity: AbilityIdentity::ImmuneBossWave,
        talent_id: 0,
        icon_id: None,
        name: "",
        description: "",
        schema: &[],
        attributes: |stats| {
            if stats.boss_wave_immune > 0 { vec![("Active", 1, AttrUnit::None)] } else { vec![] }
        },
        apply_talent: Some(|stats,_,_,_| stats.boss_wave_immune = 1),
    },

    // --- RESISTANCES ---
    Ability {
        identity: AbilityIdentity::ResistWeaken,
        talent_id: 18,
        icon_id: Some(img015::ICON_RESIST_WEAKEN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistFreeze,
        talent_id: 19,
        icon_id: Some(img015::ICON_RESIST_FREEZE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistSlow,
        talent_id: 20,
        icon_id: Some(img015::ICON_RESIST_SLOW),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistKnockback,
        talent_id: 21,
        icon_id: Some(img015::ICON_RESIST_KNOCKBACK),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistWave,
        talent_id: 22,
        icon_id: Some(img015::ICON_RESIST_WAVE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistWarp,
        talent_id: 24,
        icon_id: Some(img015::ICON_RESIST_WARP),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistCurse,
        talent_id: 30,
        icon_id: Some(img015::ICON_RESIST_CURSE),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistToxic,
        talent_id: 52,
        icon_id: Some(img015::ICON_RESIST_TOXIC),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: AbilityIdentity::ResistSurge,
        talent_id: 54,
        icon_id: Some(img015::ICON_SURGE_RESIST),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|_,_,_,_| {}),
    },

    // --- STAT TALENTS ---
    Ability {
        identity: AbilityIdentity::CostDown,
        talent_id: 25,
        icon_id: Some(img015::ICON_COST_DOWN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, reduction, _, _| stats.eoc1_cost = stats.eoc1_cost.saturating_sub(reduction)),
    },
    Ability {
        identity: AbilityIdentity::RecoverSpeedUp,
        talent_id: 26,
        icon_id: Some(img015::ICON_RECOVER_SPEED_UP),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, frames, _, _| stats.cooldown = stats.cooldown.saturating_sub(frames)),
    },
    Ability {
        identity: AbilityIdentity::MoveSpeedUp,
        talent_id: 27,
        icon_id: Some(img015::ICON_MOVE_SPEED),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, speed, _, _| stats.speed += speed),
    },
    Ability {
        identity: AbilityIdentity::AttackBuff,
        talent_id: 31,
        icon_id: Some(img015::ICON_ATTACK_BUFF),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.attack_1 = (stats.attack_1 as f32 * percentage_factor).round() as i32;
            stats.attack_2 = (stats.attack_2 as f32 * percentage_factor).round() as i32;
            stats.attack_3 = (stats.attack_3 as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: AbilityIdentity::HealthBuff,
        talent_id: 32,
        icon_id: Some(img015::ICON_HEALTH_BUFF),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.hitpoints = (stats.hitpoints as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: AbilityIdentity::TbaDown,
        talent_id: 61,
        icon_id: Some(img015::ICON_TBA_DOWN),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, percent, _, _| {
            let time_reduction = (stats.time_between_attacks as f32 * percent as f32 / 100.0).round() as i32;
            stats.time_between_attacks = stats.time_between_attacks.saturating_sub(time_reduction);
        }),
    },
    Ability {
        identity: AbilityIdentity::ImproveKnockbacks,
        talent_id: 28,
        icon_id: Some(img015::ICON_IMPROVE_KNOCKBACK_COUNT),
        name: "",
        description: "",
        schema: &[],
        attributes: |_| vec![],
        apply_talent: Some(|stats, count, _, _| stats.knockbacks += count),
    },
];