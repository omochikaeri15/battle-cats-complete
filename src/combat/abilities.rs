//! Recognition of combat abilities from raw statistic columns.
//!
//! The engine does not name a unit's abilities anywhere. Each ability is a
//! pattern across one or more statistic columns, and recognizing it means
//! knowing which columns to read and how to interpret their sentinels. This
//! module holds that knowledge as a static registry.

use crate::cat::unit::TalentGroup;
use crate::files::img015;
use crate::common::columns::Scale;

use super::{Entity, Faction};

/// The unit an ability attribute's value is expressed in.
///
/// Attribute values are bare integers whose meaning depends on their source
/// column; this tag travels alongside so a caller can render them correctly.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum AttrUnit {
    /// A bare count or flag carrying no unit.
    None,
    /// A percentage.
    Percent,
    /// A duration in frames, at thirty frames per second.
    Frames,
    /// A distance in engine distance units.
    Range,
}

/// An ability attribute's value, which may be unbounded.
///
/// Some columns use a negative value to mean the effect never expires rather
/// than a negative quantity.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum AttrValue {
    /// A bounded value, stored exactly as the engine records it.
    Finite(i32),
    /// An unbounded value, produced by the engine's negative sentinel.
    Infinite,
}

impl AttrValue {
    /// Interprets a raw column value that uses a negative unbounded sentinel.
    ///
    /// # Arguments
    /// * `raw` - The value exactly as the engine records it.
    ///
    /// # Returns
    /// An `AttrValue` that is `Infinite` when the raw value is negative, and
    /// `Finite` carrying the raw value otherwise.
    pub fn from_sentinel(raw: i32) -> Self {
        if raw < 0 { Self::Infinite } else { Self::Finite(raw) }
    }
}

impl From<i32> for AttrValue {
    fn from(raw: i32) -> Self {
        Self::Finite(raw)
    }
}

/// One named, unit-tagged quantity describing part of an ability's effect.
///
/// The three elements are the attribute's display label, its value, and the
/// unit that value is expressed in.
pub type Attribute = (&'static str, AttrValue, AttrUnit);

/// The relation between the number a talent declares and the number stored on the entity.
///
/// A few talents record the complement of the quantity they describe, so the
/// stored field and the declared value are not the same number.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Stored {
    /// The declared value is stored unchanged.
    Direct,
    /// The declared value is subtracted from the carried base before it is stored.
    Inverted(i32),
}

impl Stored {
    /// Converts a declared talent value into the number the entity carries.
    ///
    /// # Arguments
    /// * `value` - The value the talent declares at the level being applied.
    ///
    /// # Returns
    /// An `i32` holding the number stored on the entity.
    pub const fn apply(self, value: i32) -> i32 {
        match self {
            Self::Direct => value,
            Self::Inverted(base) => base - value,
        }
    }
}

/// The meaning of one `(min, max)` value pair a talent group declares.
///
/// `SkillAcquisition.csv` gives every talent group four such pairs and names
/// none of them. This describes one pair by its position: what it measures, and
/// what the engine does to it on the way into [`Entity`]. It is a different
/// list from [`Ability::schema`], which describes the attributes an ability
/// yields rather than the values a talent stores.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct TalentValue {
    /// What this pair measures.
    pub label: &'static str,
    /// The unit the pair is expressed in.
    pub unit: AttrUnit,
    /// Whether the pair is interpolated across the talent's levels, which is false when only its minimum is read.
    pub interpolated: bool,
    /// The conversion applied to the pair on the way into the entity.
    pub scale: Scale,
    /// The relation between the declared value and the stored one.
    pub stored: Stored,
}

impl TalentValue {
    const fn new(label: &'static str, unit: AttrUnit) -> Self {
        Self { label, unit, interpolated: true, scale: Scale::Raw, stored: Stored::Direct }
    }

    const fn minimum(mut self) -> Self {
        self.interpolated = false;
        self
    }

    const fn scaled(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    const fn inverted(mut self, base: i32) -> Self {
        self.stored = Stored::Inverted(base);
        self
    }
}

/// Identifies a distinct combat ability.
///
/// No single column names an ability, so this supplies the stable identifier the
/// registry and its lookups are keyed by.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Identity {
    /// Damages only the first target within the hitbox.
    SingleAttack,
    /// Damages every target within the hitbox.
    AreaAttack,
    /// Attacks in a sequence of two or three separate hits.
    MultiHit,
    /// Attacks a band of ground at a distance rather than at contact range.
    LongDistance,
    /// Attacks a band extending behind the unit as well as in front.
    OmniStrike,
    /// Carries the Red trait.
    TraitRed,
    /// Carries the Floating trait.
    TraitFloating,
    /// Carries the Black trait.
    TraitDark,
    /// Carries the Metal trait.
    TraitMetal,
    /// Carries the Traitless trait.
    TraitTraitless,
    /// Carries the Angel trait.
    TraitAngel,
    /// Carries the Alien trait.
    TraitAlien,
    /// Carries the Zombie trait.
    TraitZombie,
    /// Carries the Witch trait.
    TraitWitch,
    /// Carries the Eva Angel trait.
    TraitEva,
    /// Carries the Relic trait.
    TraitRelic,
    /// Carries the Aku trait.
    TraitAku,
    /// Carries the Dojo trait.
    TraitDojo,
    /// Carries the Starred Alien trait.
    TraitStarredAlien,
    /// Carries the Cat God trait.
    TraitCatGod,
    /// Carries the Behemoth trait.
    TraitBehemoth,
    /// Carries the Colossus trait.
    TraitColossus,
    /// Carries the Sage trait.
    TraitSage,
    /// Carries the Kaijin trait.
    TraitKaijin,
    /// Attacks without blocking the advance of opponents.
    AttackOnly,
    /// Deals increased damage to, and takes reduced damage from, its target traits.
    StrongAgainst,
    /// Deals substantially increased damage to its target traits.
    MassiveDamage,
    /// Deals the highest tier of increased damage to its target traits.
    InsaneDamage,
    /// Takes reduced damage from its target traits.
    Resist,
    /// Takes the highest tier of reduced damage from its target traits.
    InsanelyTough,
    /// Takes one damage from every hit except critical hits.
    IsMetal,
    /// Yields twice the ordinary currency when defeated.
    DoubleBounty,
    /// Deals increased damage to Zombie targets and prevents their revival.
    ZombieKiller,
    /// Strikes Zombie targets while they are burrowed.
    Soulstrike,
    /// Deals increased damage to, and takes reduced damage from, Colossus targets.
    ColossusSlayer,
    /// Deals increased damage to, and takes reduced damage from, Sage targets.
    SageSlayer,
    /// Deals increased damage to Behemoth targets and may evade their attacks.
    BehemothSlayer,
    /// Deals increased damage to, and takes reduced damage from, Witch targets.
    WitchKiller,
    /// Deals increased damage to, and takes reduced damage from, Eva Angel targets.
    EvaKiller,
    /// Deals damage proportional to a Metal target's maximum health.
    MetalKiller,
    /// Destroys a target's barrier outright.
    BarrierBreaker,
    /// Ignores a target's shield.
    ShieldPiercer,
    /// Summons a spirit copy of another unit.
    Conjure,
    /// Deals increased damage to bases.
    BaseDestroyer,
    /// Is removed from the field after attacking once.
    Kamikaze,
    /// Halts rather than attacking.
    Stop,
    /// Ignores shockwaves and prevents them passing further.
    WaveBlock,
    /// Retaliates against surges that strike it.
    CounterSurge,
    /// Emits a shockwave that travels along the ground on attack.
    WaveAttack,
    /// Emits a shockwave dealing reduced damage on attack.
    MiniWave,
    /// Creates a surge at a distant point on attack.
    SurgeAttack,
    /// Creates a surge dealing reduced damage on attack.
    MiniSurge,
    /// Creates an explosion at a distant point on attack.
    Explosion,
    /// Occasionally deals a substantial bonus strike.
    SavageBlow,
    /// Occasionally deals doubled damage that also affects Metal targets.
    CriticalHit,
    /// Gains increased attack once its health falls below a threshold.
    Strengthen,
    /// Occasionally survives a lethal hit with one health remaining.
    Survive,
    /// Occasionally becomes briefly invulnerable to its target traits.
    Dodge,
    /// Occasionally reduces a target's attack power for a period.
    Weaken,
    /// Occasionally immobilizes a target for a period.
    Freeze,
    /// Occasionally reduces a target's movement and attack rate for a period.
    Slow,
    /// Occasionally repels a target backwards.
    Knockback,
    /// Occasionally suppresses a target's trait-based abilities for a period.
    Curse,
    /// Occasionally displaces a target backwards and immobilizes it.
    Warp,
    /// An ability the source row implies but this crate does not recognize.
    Unknown,
    /// Carries a barrier absorbing damage until it is broken.
    Barrier,
    /// Carries an Aku shield absorbing damage until it is broken.
    AkuShield,
    /// Travels beneath attacks for part of its approach.
    Burrow,
    /// Returns to the field after being defeated.
    Revive,
    /// Occasionally deals damage proportional to a target's maximum health.
    Toxic,
    /// Restores health proportional to the damage it deals.
    Drain,
    /// Creates a surge at a distant point when defeated.
    DeathSurge,
    /// Is wholly unaffected by shockwaves.
    ImmuneWave,
    /// Is wholly unaffected by surges.
    ImmuneSurge,
    /// Is wholly unaffected by explosions.
    ImmuneExplosion,
    /// Is wholly unaffected by the weaken effect.
    ImmuneWeaken,
    /// Is wholly unaffected by the freeze effect.
    ImmuneFreeze,
    /// Is wholly unaffected by the slow effect.
    ImmuneSlow,
    /// Is wholly unaffected by knockback.
    ImmuneKnockback,
    /// Is wholly unaffected by the curse effect.
    ImmuneCurse,
    /// Is wholly unaffected by toxic damage.
    ImmuneToxic,
    /// Is wholly unaffected by drain effects.
    ImmuneDrain,
    /// Is wholly unaffected by the warp effect.
    ImmuneWarp,
    /// Is wholly unaffected by the boss wave knockback.
    ImmuneBossWave,
    /// Has the duration or strength of the weaken effect reduced against it.
    ResistWeaken,
    /// Has the duration or strength of the freeze effect reduced against it.
    ResistFreeze,
    /// Has the duration or strength of the slow effect reduced against it.
    ResistSlow,
    /// Has the duration or strength of knockback reduced against it.
    ResistKnockback,
    /// Has the duration or strength of shockwaves reduced against it.
    ResistWave,
    /// Has the duration or strength of the warp effect reduced against it.
    ResistWarp,
    /// Has the duration or strength of the curse effect reduced against it.
    ResistCurse,
    /// Has the duration or strength of toxic damage reduced against it.
    ResistToxic,
    /// Has the duration or strength of surges reduced against it.
    ResistSurge,
    /// Reduces its own deployment cost.
    CostDown,
    /// Reduces its own redeployment delay.
    RecoverSpeedUp,
    /// Increases its own movement rate.
    MoveSpeedUp,
    /// Increases its own attack power.
    AttackBuff,
    /// Increases its own health pool.
    HealthBuff,
    /// Reduces its own delay between attacks.
    TbaDown,
    /// Increases the number of times it may be repelled before being defeated.
    ImproveKnockbacks,
    /// Lingers on the field for a delay after being defeated before its corpse is removed.
    TimeBeforeDeath,
}

/// The complete definition of one combat ability.
///
/// An ability is a pattern across several statistic columns rather than a stored
/// value, so its definition carries the logic to extract it. Holding that as a
/// function pointer keeps the registry in immutable static memory.
pub struct Ability {
    /// The stable identifier this ability is keyed by.
    pub identity: Identity,
    /// The talent identifier that grants this ability, when it can be granted as a talent.
    pub talent_id: Option<u8>,
    /// The sprite index of this ability's icon in the `img015` atlas, when it has one.
    pub icon_id: Option<usize>,
    /// The one-based line of `nyankoPictureBook2` that names this ability for cats, when the Cat dictionary lists it.
    pub cat_glossary: Option<u8>,
    /// The one-based line of `EnemyPictureBook2` that names this ability for enemies, when the Enemy dictionary lists it.
    pub enemy_glossary: Option<u8>,
    /// The zero-based row of `attribute_explonation` that names this trait, when the Talent Orb screen lists it.
    pub orb_trait: Option<u8>,
    /// The labels and units of the attributes `attributes` produces, in the same order.
    pub schema: &'static [(&'static str, AttrUnit)],
    /// One entry per `(min, max)` value pair `apply_talent` consumes, in pair order.
    pub talent_values: &'static [TalentValue],
    /// Extracts this ability's attributes from an entity, yielding an empty vector when the entity lacks the ability.
    pub attributes: fn(&Entity) -> Vec<Attribute>,
    /// Applies this ability to an entity as a talent upgrade, when it can be granted as one.
    pub apply_talent: Option<fn(&mut Entity, val1: i32, val2: i32, group: &TalentGroup)>,
}

fn active() -> Vec<Attribute> {
    vec![("Active", AttrValue::Finite(1), AttrUnit::None)]
}

fn flag(raw: i32) -> Vec<Attribute> {
    if raw > 0 { active() } else { Vec::new() }
}

fn get_dur_val(v1: i32, v2: i32) -> i32 {
    if v1 != 0 { v1 } else { v2 }
}

fn wave_reach(stats: &Entity) -> i32 {
    let base = match stats.faction {
        Faction::Cat => 332.5,
        Faction::Enemy => 467.5,
    };
    (base + ((stats.wave_level - 1) as f32 * 200.0)).round() as i32
}

fn has_omni(stats: &Entity) -> bool {
    (stats.long_distance_1_span < 0 || (stats.long_distance_1_span == 0 && stats.long_distance_1_anchor != 0)) ||
        (stats.long_distance_2_flag > 0 && (stats.long_distance_2_span < 0 || (stats.long_distance_2_span == 0 && stats.long_distance_2_anchor != 0))) ||
        (stats.long_distance_3_flag > 0 && (stats.long_distance_3_span < 0 || (stats.long_distance_3_span == 0 && stats.long_distance_3_anchor != 0)))
}

fn has_long_distance(stats: &Entity) -> bool {
    (stats.long_distance_1_span > 0) ||
        (stats.long_distance_2_flag > 0 && stats.long_distance_2_span > 0) ||
        (stats.long_distance_3_flag > 0 && stats.long_distance_3_span > 0)
}

/// Looks up the ability a talent identifier grants.
///
/// # Arguments
/// * `id` - The talent identifier recorded on a talent group.
///
/// # Returns
/// An `Option` containing a reference to the matching `Ability`, or `None` when
/// no ability is granted by that talent identifier.
pub fn get_talent(id: u8) -> Option<&'static Ability> {
    REGISTRY.iter().find(|ability| ability.talent_id == Some(id))
}

/// Looks up an ability's definition by its identifier.
///
/// # Arguments
/// * `identity` - The ability to retrieve.
///
/// # Returns
/// An `Option` containing a reference to the matching `Ability`, or `None` when
/// the registry carries no definition for it.
pub fn get_ability(identity: Identity) -> Option<&'static Ability> {
    REGISTRY.iter().find(|ability| ability.identity == identity)
}

/// The definitions of every combat ability this crate recognizes.
///
/// Prefer [`get_ability`] and [`get_talent`] over scanning this directly; the
/// ordering of the entries is not part of the interface and may change.
pub static REGISTRY: &[Ability] = &[
    Ability {
        identity: Identity::SingleAttack,
        talent_id: None,
        icon_id: Some(img015::ICON_SINGLE_ATTACK),
        cat_glossary: Some(39),
        enemy_glossary: Some(39),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| if stats.area_attack == 0 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::AreaAttack,
        talent_id: None,
        icon_id: Some(img015::ICON_AREA_ATTACK),
        cat_glossary: Some(33),
        enemy_glossary: Some(33),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| if stats.area_attack == 1 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::MultiHit,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.attack_2_damage),
        apply_talent: None,
    },
    Ability {
        identity: Identity::LongDistance,
        talent_id: None,
        icon_id: Some(img015::ICON_LONG_DISTANCE),
        cat_glossary: Some(34),
        enemy_glossary: Some(34),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| {
            if has_long_distance(stats) && !has_omni(stats) { active() } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::OmniStrike,
        talent_id: None,
        icon_id: Some(img015::ICON_OMNI_STRIKE),
        cat_glossary: Some(45),
        enemy_glossary: Some(45),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| if has_omni(stats) { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitRed,
        talent_id: Some(33),
        icon_id: Some(img015::ICON_TRAIT_RED),
        cat_glossary: Some(64),
        enemy_glossary: Some(1),
        orb_trait: Some(0),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_red),
        apply_talent: Some(|stats,_,_,_| stats.trait_red = 1),
    },
    Ability {
        identity: Identity::TraitFloating,
        talent_id: Some(34),
        icon_id: Some(img015::ICON_TRAIT_FLOATING),
        cat_glossary: Some(65),
        enemy_glossary: Some(2),
        orb_trait: Some(1),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_floating),
        apply_talent: Some(|stats,_,_,_| stats.trait_floating = 1),
    },
    Ability {
        identity: Identity::TraitDark,
        talent_id: Some(35),
        icon_id: Some(img015::ICON_TRAIT_BLACK),
        cat_glossary: Some(66),
        enemy_glossary: Some(3),
        orb_trait: Some(2),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_dark),
        apply_talent: Some(|stats,_,_,_| stats.trait_dark = 1),
    },
    Ability {
        identity: Identity::TraitMetal,
        talent_id: Some(36),
        icon_id: Some(img015::ICON_TRAIT_METAL),
        cat_glossary: Some(67),
        enemy_glossary: Some(4),
        orb_trait: Some(3),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_metal),
        apply_talent: Some(|stats,_,_,_| stats.trait_metal = 1),
    },
    Ability {
        identity: Identity::TraitAngel,
        talent_id: Some(37),
        icon_id: Some(img015::ICON_TRAIT_ANGEL),
        cat_glossary: Some(68),
        enemy_glossary: Some(5),
        orb_trait: Some(4),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_angel),
        apply_talent: Some(|stats,_,_,_| stats.trait_angel = 1),
    },
    Ability {
        identity: Identity::TraitAlien,
        talent_id: Some(38),
        icon_id: Some(img015::ICON_TRAIT_ALIEN),
        cat_glossary: Some(69),
        enemy_glossary: Some(6),
        orb_trait: Some(5),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_alien),
        apply_talent: Some(|stats,_,_,_| stats.trait_alien = 1),
    },
    Ability {
        identity: Identity::TraitZombie,
        talent_id: Some(39),
        icon_id: Some(img015::ICON_TRAIT_ZOMBIE),
        cat_glossary: Some(70),
        enemy_glossary: Some(7),
        orb_trait: Some(6),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_zombie),
        apply_talent: Some(|stats,_,_,_| stats.trait_zombie = 1),
    },
    Ability {
        identity: Identity::TraitRelic,
        talent_id: Some(40),
        icon_id: Some(img015::ICON_TRAIT_RELIC),
        cat_glossary: Some(71),
        enemy_glossary: Some(8),
        orb_trait: Some(7),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_relic),
        apply_talent: Some(|stats,_,_,_| stats.trait_relic = 1),
    },
    Ability {
        identity: Identity::TraitAku,
        talent_id: Some(57),
        icon_id: Some(img015::ICON_TRAIT_AKU),
        cat_glossary: Some(83),
        enemy_glossary: Some(9),
        orb_trait: Some(11),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_aku),
        apply_talent: Some(|stats,_,_,_| stats.trait_aku = 1),
    },
    Ability {
        identity: Identity::TraitTraitless,
        talent_id: Some(41),
        icon_id: Some(img015::ICON_TRAIT_TRAITLESS),
        cat_glossary: Some(72),
        enemy_glossary: Some(10),
        orb_trait: Some(8),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_traitless),
        apply_talent: Some(|stats,_,_,_| stats.trait_traitless = 1),
    },
    Ability {
        identity: Identity::TraitWitch,
        talent_id: None,
        icon_id: Some(img015::ICON_WITCH),
        cat_glossary: None,
        enemy_glossary: Some(14),
        orb_trait: Some(9),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_witch),
        apply_talent: Some(|stats,_,_,_| stats.trait_witch = 1),
    },
    Ability {
        identity: Identity::TraitEva,
        talent_id: None,
        icon_id: Some(img015::ICON_EVA),
        cat_glossary: None,
        enemy_glossary: Some(15),
        orb_trait: Some(10),
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_eva),
        apply_talent: Some(|stats,_,_,_| stats.trait_eva = 1),
    },
    Ability {
        identity: Identity::TraitDojo,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_dojo),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitStarredAlien,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| if stats.trait_starred_alien == 1 { active() } else { Vec::new() },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitCatGod,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Type", AttrUnit::None)],
        talent_values: &[],
        attributes: |stats| {
            if stats.trait_starred_alien >= 2 && stats.trait_starred_alien <= 4 {
                vec![("Type", AttrValue::Finite(stats.trait_starred_alien), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitColossus,
        talent_id: None,
        icon_id: Some(img015::ICON_COLOSSUS),
        cat_glossary: None,
        enemy_glossary: Some(11),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_colossus),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitBehemoth,
        talent_id: None,
        icon_id: Some(img015::ICON_BEHEMOTH),
        cat_glossary: None,
        enemy_glossary: Some(12),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_behemoth),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitSage,
        talent_id: None,
        icon_id: Some(img015::ICON_SAGE),
        cat_glossary: None,
        enemy_glossary: Some(13),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_sage),
        apply_talent: None,
    },
    Ability {
        identity: Identity::TraitKaijin,
        talent_id: None,
        icon_id: Some(img015::ICON_SUPERVILLIAN),
        cat_glossary: None,
        enemy_glossary: Some(16),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.trait_kaijin),
        apply_talent: None,
    },
    Ability {
        identity: Identity::AttackOnly,
        talent_id: Some(4),
        icon_id: Some(img015::ICON_ATTACK_ONLY),
        cat_glossary: Some(24),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.attack_only),
        apply_talent: Some(|stats, _, _, _| stats.attack_only = 1),
    },
    Ability {
        identity: Identity::StrongAgainst,
        talent_id: Some(5),
        icon_id: Some(img015::ICON_STRONG_AGAINST),
        cat_glossary: Some(25),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.strong_against),
        apply_talent: Some(|stats, _, _, _| stats.strong_against = 1),
    },
    Ability {
        identity: Identity::MassiveDamage,
        talent_id: Some(7),
        icon_id: Some(img015::ICON_MASSIVE_DAMAGE),
        cat_glossary: Some(28),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.massive_damage),
        apply_talent: Some(|stats, _, _, _| stats.massive_damage = 1),
    },
    Ability {
        identity: Identity::InsaneDamage,
        talent_id: Some(7),
        icon_id: Some(img015::ICON_INSANE_DAMAGE),
        cat_glossary: Some(62),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.insane_damage),
        apply_talent: None,
    },
    Ability {
        identity: Identity::Resist,
        talent_id: Some(6),
        icon_id: Some(img015::ICON_RESIST),
        cat_glossary: Some(26),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.resist),
        apply_talent: Some(|stats, _, _, _| stats.resist = 1),
    },
    Ability {
        identity: Identity::InsanelyTough,
        talent_id: Some(6),
        icon_id: Some(img015::ICON_INSANELY_TOUGH),
        cat_glossary: Some(63),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.insanely_tough),
        apply_talent: None,
    },
    Ability {
        identity: Identity::IsMetal,
        talent_id: Some(43),
        icon_id: Some(img015::ICON_METAL),
        cat_glossary: Some(31),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.is_metal),
        apply_talent: Some(|stats,_,_,_| stats.is_metal = 1),
    },
    Ability {
        identity: Identity::DoubleBounty,
        talent_id: Some(16),
        icon_id: Some(img015::ICON_DOUBLE_BOUNTY),
        cat_glossary: Some(27),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.double_bounty),
        apply_talent: Some(|stats, _, _, _| stats.double_bounty = 1),
    },
    Ability {
        identity: Identity::ZombieKiller,
        talent_id: Some(14),
        icon_id: Some(img015::ICON_ZOMBIE_KILLER),
        cat_glossary: Some(41),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.zombie_killer),
        apply_talent: Some(|stats, _, _, _| stats.zombie_killer = 1),
    },
    Ability {
        identity: Identity::Soulstrike,
        talent_id: Some(59),
        icon_id: Some(img015::ICON_SOULSTRIKE),
        cat_glossary: Some(85),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| {
            if stats.soulstrike == 2 || (stats.soulstrike > 0 && stats.zombie_killer > 0) { active() } else { Vec::new() }
        },
        apply_talent: Some(|stats, _, _, _| stats.soulstrike = 2),
    },
    Ability {
        identity: Identity::ColossusSlayer,
        talent_id: Some(63),
        icon_id: Some(img015::ICON_COLOSSUS_SLAYER),
        cat_glossary: Some(84),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.colossus_slayer),
        apply_talent: Some(|stats, _, _, _| stats.colossus_slayer = 1),
    },
    Ability {
        identity: Identity::SageSlayer,
        talent_id: Some(66),
        icon_id: Some(img015::ICON_SAGE_SLAYER),
        cat_glossary: Some(91),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.sage_slayer),
        apply_talent: Some(|stats, _, _, _| stats.sage_slayer = 1),
    },
    Ability {
        identity: Identity::BehemothSlayer,
        talent_id: Some(64),
        icon_id: Some(img015::ICON_BEHEMOTH_SLAYER),
        cat_glossary: Some(86),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[
            ("Dodge Chance", AttrUnit::Percent),
            ("Dodge Duration", AttrUnit::Frames),
        ],
        talent_values: &[TalentValue::new("Dodge Chance", AttrUnit::Percent), TalentValue::new("Dodge Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.behemoth_slayer <= 0 { return Vec::new(); }
            if stats.behemoth_dodge_chance > 0 {
                vec![
                    ("Active", AttrValue::Finite(1), AttrUnit::None),
                    ("Dodge Chance", AttrValue::Finite(stats.behemoth_dodge_chance), AttrUnit::Percent),
                    ("Dodge Duration", AttrValue::Finite(stats.behemoth_dodge_duration), AttrUnit::Frames),
                ]
            } else {
                active()
            }
        },
        apply_talent: Some(|stats, chance, duration, _| {
            stats.behemoth_slayer = 1;
            stats.behemoth_dodge_chance = if chance > 0 { chance } else { 5 };
            stats.behemoth_dodge_duration = if duration > 0 { duration } else { 30 };
        }),
    },
    Ability {
        identity: Identity::WitchKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_WITCH_KILLER),
        cat_glossary: Some(40),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.witch_killer),
        apply_talent: Some(|stats,_,_,_| stats.witch_killer = 1),
    },
    Ability {
        identity: Identity::EvaKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_EVA_KILLER),
        cat_glossary: Some(46),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.eva_killer),
        apply_talent: Some(|stats,_,_,_| stats.eva_killer = 1),
    },
    Ability {
        identity: Identity::MetalKiller,
        talent_id: None,
        icon_id: Some(img015::ICON_METAL_KILLER),
        cat_glossary: Some(92),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Damage", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Damage", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.metal_killer_percent > 0 {
                vec![("Damage", AttrValue::Finite(stats.metal_killer_percent), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, percent, _, _| stats.metal_killer_percent = percent),
    },
    Ability {
        identity: Identity::BarrierBreaker,
        talent_id: Some(15),
        icon_id: Some(img015::ICON_BARRIER_BREAKER),
        cat_glossary: Some(43),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.barrier_breaker_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.barrier_breaker_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.barrier_breaker_chance += chance),
    },
    Ability {
        identity: Identity::ShieldPiercer,
        talent_id: Some(58),
        icon_id: Some(img015::ICON_SHIELD_PIERCER),
        cat_glossary: Some(82),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.shield_pierce_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.shield_pierce_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.shield_pierce_chance += chance),
    },
    Ability {
        identity: Identity::Conjure,
        talent_id: None,
        icon_id: Some(img015::ICON_CONJURE),
        cat_glossary: Some(90),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Spirit ID", AttrUnit::None)],
        talent_values: &[],
        attributes: |stats| {
            if stats.conjure_unit_id > -1 {
                vec![("Spirit ID", AttrValue::Finite(stats.conjure_unit_id), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::BaseDestroyer,
        talent_id: Some(12),
        icon_id: Some(img015::ICON_BASE_DESTROYER),
        cat_glossary: Some(22),
        enemy_glossary: Some(22),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.base_destroyer),
        apply_talent: Some(|stats, _, _, _| stats.base_destroyer = 1),
    },
    Ability {
        identity: Identity::Kamikaze,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Attacks", AttrUnit::None)],
        talent_values: &[],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 2 {
                vec![("Attacks", AttrValue::Finite(stats.attack_count_total), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Stop,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Attacks", AttrUnit::None)],
        talent_values: &[],
        attributes: |stats| {
            if stats.attack_count_total > -1 && stats.attack_count_state == 0 {
                vec![("Attacks", AttrValue::Finite(stats.attack_count_total), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::WaveBlock,
        talent_id: None,
        icon_id: Some(img015::ICON_WAVE_BLOCK),
        cat_glossary: Some(42),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.wave_block),
        apply_talent: Some(|stats, _, _, _| stats.wave_block = 1),
    },
    Ability {
        identity: Identity::CounterSurge,
        talent_id: Some(68),
        icon_id: Some(img015::ICON_COUNTER_SURGE),
        cat_glossary: Some(89),
        enemy_glossary: Some(89),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.counter_surge),
        apply_talent: Some(|stats,_,_,_| stats.counter_surge = 1),
    },
    Ability {
        identity: Identity::WaveAttack,
        talent_id: Some(17),
        icon_id: Some(img015::ICON_WAVE),
        cat_glossary: Some(30),
        enemy_glossary: Some(30),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Max Reach", AttrUnit::Range),
        ],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Level", AttrUnit::None)],
        attributes: |stats| {
            if stats.mini_wave_flag == 0 && stats.wave_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.wave_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.wave_level), AttrUnit::None),
                    ("Max Reach", AttrValue::Finite(wave_reach(stats)), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: Identity::MiniWave,
        talent_id: Some(62),
        icon_id: Some(img015::ICON_MINI_WAVE),
        cat_glossary: Some(79),
        enemy_glossary: Some(79),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Max Reach", AttrUnit::Range),
        ],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Level", AttrUnit::None)],
        attributes: |stats| {
            if stats.mini_wave_flag > 0 && stats.wave_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.wave_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.wave_level), AttrUnit::None),
                    ("Max Reach", AttrValue::Finite(wave_reach(stats)), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, _| { stats.mini_wave_flag = 1; stats.wave_chance += chance; stats.wave_level = level; }),
    },
    Ability {
        identity: Identity::SurgeAttack,
        talent_id: Some(56),
        icon_id: Some(img015::ICON_SURGE),
        cat_glossary: Some(75),
        enemy_glossary: Some(75),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        talent_values: &[
            TalentValue::new("Chance", AttrUnit::Percent),
            TalentValue::new("Level", AttrUnit::None),
            TalentValue::new("Min Range", AttrUnit::Range).minimum().scaled(Scale::Quarter),
            TalentValue::new("Width", AttrUnit::Range).minimum().scaled(Scale::Quarter),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag == 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.surge_spawn_anchor + stats.surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.surge_chance += chance;
            stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::MiniSurge,
        talent_id: Some(65),
        icon_id: Some(img015::ICON_MINI_SURGE),
        cat_glossary: Some(88),
        enemy_glossary: Some(88),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        talent_values: &[
            TalentValue::new("Chance", AttrUnit::Percent),
            TalentValue::new("Level", AttrUnit::None),
            TalentValue::new("Min Range", AttrUnit::Range).minimum().scaled(Scale::Quarter),
            TalentValue::new("Width", AttrUnit::Range).minimum().scaled(Scale::Quarter),
        ],
        attributes: |stats| {
            if stats.mini_surge_flag > 0 && stats.surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.surge_spawn_anchor + stats.surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, level, group_data| {
            stats.mini_surge_flag = 1;
            stats.surge_chance += chance;
            stats.surge_level = level;
            stats.surge_spawn_anchor = group_data.min_3 as i32 / 4;
            stats.surge_spawn_span = group_data.min_4 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::Explosion,
        talent_id: Some(67),
        icon_id: Some(img015::ICON_EXPLOSION),
        cat_glossary: Some(98),
        enemy_glossary: Some(98),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        talent_values: &[
            TalentValue::new("Chance", AttrUnit::Percent),
            TalentValue::new("Min Range", AttrUnit::Range).minimum().scaled(Scale::Quarter),
            TalentValue::new("Width", AttrUnit::Range).minimum().scaled(Scale::Quarter),
        ],
        attributes: |stats| {
            if stats.explosion_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.explosion_chance), AttrUnit::Percent),
                    ("Min Range", AttrValue::Finite(stats.explosion_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.explosion_spawn_anchor + stats.explosion_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.explosion_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, group_data| {
            stats.explosion_chance += chance;
            stats.explosion_spawn_anchor = group_data.min_2 as i32 / 4;
            stats.explosion_spawn_span = group_data.min_3 as i32 / 4;
        }),
    },
    Ability {
        identity: Identity::SavageBlow,
        talent_id: Some(50),
        icon_id: Some(img015::ICON_SAVAGE_BLOW),
        cat_glossary: Some(73),
        enemy_glossary: Some(73),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.savage_blow_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.savage_blow_chance), AttrUnit::Percent),
                    ("Boost", AttrValue::Finite(stats.savage_blow_boost), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, boost, _| {
            stats.savage_blow_chance += chance;
            if boost > 0 { stats.savage_blow_boost = boost; }
        }),
    },
    Ability {
        identity: Identity::CriticalHit,
        talent_id: Some(13),
        icon_id: Some(img015::ICON_CRITICAL_HIT),
        cat_glossary: Some(23),
        enemy_glossary: Some(23),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.critical_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.critical_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.critical_chance += chance),
    },
    Ability {
        identity: Identity::Strengthen,
        talent_id: Some(10),
        icon_id: Some(img015::ICON_STRENGTHEN),
        cat_glossary: Some(18),
        enemy_glossary: Some(18),
        orb_trait: None,
        schema: &[("HP", AttrUnit::Percent), ("Boost", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("HP", AttrUnit::Percent).inverted(100), TalentValue::new("Boost", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.strengthen_threshold > 0 {
                vec![
                    ("HP", AttrValue::Finite(stats.strengthen_threshold), AttrUnit::Percent),
                    ("Boost", AttrValue::Finite(stats.strengthen_boost), AttrUnit::Percent),
                ]
            } else { Vec::new() }
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
        identity: Identity::Survive,
        talent_id: Some(11),
        icon_id: Some(img015::ICON_SURVIVE),
        cat_glossary: Some(21),
        enemy_glossary: Some(21),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.survive > 0 {
                vec![("Chance", AttrValue::Finite(stats.survive), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.survive += chance),
    },
    Ability {
        identity: Identity::Dodge,
        talent_id: Some(51),
        icon_id: Some(img015::ICON_DODGE),
        cat_glossary: Some(74),
        enemy_glossary: Some(74),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.dodge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.dodge_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.dodge_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, _| { stats.dodge_chance += chance; stats.dodge_duration += duration; }),
    },
    Ability {
        identity: Identity::Weaken,
        talent_id: Some(1),
        icon_id: Some(img015::ICON_WEAKEN),
        cat_glossary: Some(17),
        enemy_glossary: Some(17),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Reduced To", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        talent_values: &[
            TalentValue::new("Chance", AttrUnit::Percent),
            TalentValue::new("Duration", AttrUnit::Frames),
            TalentValue::new("Reduced To", AttrUnit::Percent).minimum().inverted(100),
        ],
        attributes: |stats| {
            if stats.weaken_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.weaken_chance), AttrUnit::Percent),
                    ("Reduced To", AttrValue::Finite(stats.weaken_to), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.weaken_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.weaken_chance == 0 {
                stats.weaken_chance = chance;
                stats.weaken_duration = duration;
                stats.weaken_to = (100 - group_data.min_3) as i32;
            } else if group_data.text_id == 42 {
                stats.weaken_duration += get_dur_val(chance, duration);
            } else {
                stats.weaken_chance += chance;
                stats.weaken_duration += duration;
            }
        }),
    },
    Ability {
        identity: Identity::Freeze,
        talent_id: Some(2),
        icon_id: Some(img015::ICON_FREEZE),
        cat_glossary: Some(19),
        enemy_glossary: Some(19),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.freeze_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.freeze_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.freeze_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.freeze_chance == 0 {
                stats.freeze_chance = chance;
                stats.freeze_duration = duration;
            } else if group_data.text_id == 74 {
                stats.freeze_chance += chance;
            } else {
                stats.freeze_duration += get_dur_val(chance, duration);
            }
        }),
    },
    Ability {
        identity: Identity::Slow,
        talent_id: Some(3),
        icon_id: Some(img015::ICON_SLOW),
        cat_glossary: Some(20),
        enemy_glossary: Some(20),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.slow_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.slow_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.slow_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.slow_chance == 0 {
                stats.slow_chance = chance;
                stats.slow_duration = duration;
            } else if group_data.text_id == 63 {
                stats.slow_chance += chance;
            } else {
                stats.slow_duration += get_dur_val(chance, duration);
            }
        }),
    },
    Ability {
        identity: Identity::Knockback,
        talent_id: Some(8),
        icon_id: Some(img015::ICON_KNOCKBACK),
        cat_glossary: Some(29),
        enemy_glossary: Some(29),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent)],
        attributes: |stats| {
            if stats.knockback_chance > 0 {
                vec![("Chance", AttrValue::Finite(stats.knockback_chance), AttrUnit::Percent)]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, _, _| stats.knockback_chance += chance),
    },
    Ability {
        identity: Identity::Curse,
        talent_id: Some(60),
        icon_id: Some(img015::ICON_CURSE),
        cat_glossary: Some(77),
        enemy_glossary: Some(77),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames)],
        talent_values: &[TalentValue::new("Chance", AttrUnit::Percent), TalentValue::new("Duration", AttrUnit::Frames)],
        attributes: |stats| {
            if stats.curse_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.curse_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.curse_duration), AttrUnit::Frames),
                ]
            } else { Vec::new() }
        },
        apply_talent: Some(|stats, chance, duration, group_data| {
            if stats.curse_chance == 0 {
                stats.curse_chance = chance;
                stats.curse_duration = duration;
            } else if group_data.text_id == 93 {
                stats.curse_duration += get_dur_val(chance, duration);
            } else {
                stats.curse_chance += chance;
            }
        }),
    },
    Ability {
        identity: Identity::Warp,
        talent_id: Some(9),
        icon_id: Some(img015::ICON_WARP),
        cat_glossary: Some(48),
        enemy_glossary: Some(48),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Duration", AttrUnit::Frames), ("Min Distance", AttrUnit::Range), ("Max Distance", AttrUnit::Range)],
        talent_values: &[],
        attributes: |stats| {
            if stats.warp_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.warp_chance), AttrUnit::Percent),
                    ("Duration", AttrValue::Finite(stats.warp_duration), AttrUnit::Frames),
                    ("Min Distance", AttrValue::Finite(stats.warp_distance_anchor), AttrUnit::Range),
                    ("Max Distance", AttrValue::Finite(stats.warp_distance_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Unknown,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.has_unknown_abilities),
        apply_talent: None,
    },
    Ability {
        identity: Identity::Barrier,
        talent_id: None,
        icon_id: Some(img015::ICON_BARRIER),
        cat_glossary: None,
        enemy_glossary: Some(96),
        orb_trait: None,
        schema: &[("Hitpoints", AttrUnit::None)],
        talent_values: &[],
        attributes: |stats| {
            if stats.barrier_hitpoints > 0 {
                vec![("Hitpoints", AttrValue::Finite(stats.barrier_hitpoints), AttrUnit::None)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::AkuShield,
        talent_id: None,
        icon_id: Some(img015::ICON_SHIELD),
        cat_glossary: None,
        enemy_glossary: Some(97),
        orb_trait: None,
        schema: &[("Hitpoints", AttrUnit::None), ("Regen", AttrUnit::Percent)],
        talent_values: &[],
        attributes: |stats| {
            if stats.shield_hitpoints > 0 {
                vec![
                    ("Hitpoints", AttrValue::Finite(stats.shield_hitpoints), AttrUnit::None),
                    ("Regen", AttrValue::Finite(stats.shield_regen), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Burrow,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Count", AttrUnit::None), ("Distance", AttrUnit::Range)],
        talent_values: &[],
        attributes: |stats| {
            if stats.burrow_amount != 0 {
                vec![
                    ("Count", AttrValue::from_sentinel(stats.burrow_amount), AttrUnit::None),
                    ("Distance", AttrValue::Finite(stats.burrow_distance), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Revive,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Count", AttrUnit::None), ("Duration", AttrUnit::Frames), ("Hitpoints", AttrUnit::Percent)],
        talent_values: &[],
        attributes: |stats| {
            if stats.revive_count != 0 {
                vec![
                    ("Count", AttrValue::from_sentinel(stats.revive_count), AttrUnit::None),
                    ("Duration", AttrValue::Finite(stats.revive_time), AttrUnit::Frames),
                    ("Hitpoints", AttrValue::Finite(stats.revive_hp), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Toxic,
        talent_id: None,
        icon_id: Some(img015::ICON_TOXIC),
        cat_glossary: None,
        enemy_glossary: Some(94),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Damage", AttrUnit::Percent)],
        talent_values: &[],
        attributes: |stats| {
            if stats.toxic_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.toxic_chance), AttrUnit::Percent),
                    ("Damage", AttrValue::Finite(stats.toxic_damage), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::Drain,
        talent_id: None,
        icon_id: Some(img015::ICON_DRAIN),
        cat_glossary: None,
        enemy_glossary: Some(100),
        orb_trait: None,
        schema: &[("Chance", AttrUnit::Percent), ("Amount", AttrUnit::Percent)],
        talent_values: &[],
        attributes: |stats| {
            if stats.drain_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.drain_chance), AttrUnit::Percent),
                    ("Amount", AttrValue::Finite(stats.drain_percent), AttrUnit::Percent),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::DeathSurge,
        talent_id: None,
        icon_id: Some(img015::ICON_DEATH_SURGE),
        cat_glossary: None,
        enemy_glossary: Some(95),
        orb_trait: None,
        schema: &[
            ("Chance", AttrUnit::Percent),
            ("Level", AttrUnit::None),
            ("Min Range", AttrUnit::Range),
            ("Max Range", AttrUnit::Range),
            ("Width", AttrUnit::Range),
        ],
        talent_values: &[],
        attributes: |stats| {
            if stats.death_surge_chance > 0 {
                vec![
                    ("Chance", AttrValue::Finite(stats.death_surge_chance), AttrUnit::Percent),
                    ("Level", AttrValue::Finite(stats.death_surge_level), AttrUnit::None),
                    ("Min Range", AttrValue::Finite(stats.death_surge_spawn_anchor), AttrUnit::Range),
                    ("Max Range", AttrValue::Finite(stats.death_surge_spawn_anchor + stats.death_surge_spawn_span), AttrUnit::Range),
                    ("Width", AttrValue::Finite(stats.death_surge_spawn_span), AttrUnit::Range),
                ]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
    Ability {
        identity: Identity::ImmuneWave,
        talent_id: Some(48),
        icon_id: Some(img015::ICON_IMMUNE_WAVE),
        cat_glossary: Some(32),
        enemy_glossary: Some(32),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.wave_immune),
        apply_talent: Some(|stats,_,_,_| stats.wave_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneSurge,
        talent_id: Some(55),
        icon_id: Some(img015::ICON_IMMUNE_SURGE),
        cat_glossary: Some(78),
        enemy_glossary: Some(78),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.surge_immune),
        apply_talent: Some(|stats,_,_,_| stats.surge_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneExplosion,
        talent_id: Some(69),
        icon_id: Some(img015::ICON_IMMUNE_EXPLOSION),
        cat_glossary: Some(99),
        enemy_glossary: Some(99),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.explosion_immune),
        apply_talent: Some(|stats,_,_,_| stats.explosion_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneWeaken,
        talent_id: Some(44),
        icon_id: Some(img015::ICON_IMMUNE_WEAKEN),
        cat_glossary: Some(35),
        enemy_glossary: Some(35),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.weaken_immune),
        apply_talent: Some(|stats,_,_,_| stats.weaken_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneFreeze,
        talent_id: Some(45),
        icon_id: Some(img015::ICON_IMMUNE_FREEZE),
        cat_glossary: Some(36),
        enemy_glossary: Some(36),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.freeze_immune),
        apply_talent: Some(|stats,_,_,_| stats.freeze_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneSlow,
        talent_id: Some(46),
        icon_id: Some(img015::ICON_IMMUNE_SLOW),
        cat_glossary: Some(37),
        enemy_glossary: Some(37),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.slow_immune),
        apply_talent: Some(|stats,_,_,_| stats.slow_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneKnockback,
        talent_id: Some(47),
        icon_id: Some(img015::ICON_IMMUNE_KNOCKBACK),
        cat_glossary: Some(38),
        enemy_glossary: Some(38),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.knockback_immune),
        apply_talent: Some(|stats,_,_,_| stats.knockback_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneCurse,
        talent_id: Some(29),
        icon_id: Some(img015::ICON_IMMUNE_CURSE),
        cat_glossary: Some(47),
        enemy_glossary: Some(47),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.curse_immune),
        apply_talent: Some(|stats,_,_,_| stats.curse_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneWarp,
        talent_id: Some(49),
        icon_id: Some(img015::ICON_IMMUNE_WARP),
        cat_glossary: Some(44),
        enemy_glossary: Some(44),
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.warp_immune),
        apply_talent: Some(|stats,_,_,_| stats.warp_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneToxic,
        talent_id: Some(53),
        icon_id: Some(img015::ICON_IMMUNE_TOXIC),
        cat_glossary: Some(76),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.toxic_immune),
        apply_talent: Some(|stats,_,_,_| stats.toxic_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneDrain,
        talent_id: None,
        icon_id: Some(img015::ICON_IMMUNE_DRAIN),
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.drain_immune),
        apply_talent: Some(|stats,_,_,_| stats.drain_immune = 1),
    },
    Ability {
        identity: Identity::ImmuneBossWave,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |stats| flag(stats.boss_wave_immune),
        apply_talent: Some(|stats,_,_,_| stats.boss_wave_immune = 1),
    },
    Ability {
        identity: Identity::ResistWeaken,
        talent_id: Some(18),
        icon_id: Some(img015::ICON_RESIST_WEAKEN),
        cat_glossary: Some(49),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistFreeze,
        talent_id: Some(19),
        icon_id: Some(img015::ICON_RESIST_FREEZE),
        cat_glossary: Some(50),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistSlow,
        talent_id: Some(20),
        icon_id: Some(img015::ICON_RESIST_SLOW),
        cat_glossary: Some(51),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistKnockback,
        talent_id: Some(21),
        icon_id: Some(img015::ICON_RESIST_KNOCKBACK),
        cat_glossary: Some(52),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistWave,
        talent_id: Some(22),
        icon_id: Some(img015::ICON_RESIST_WAVE),
        cat_glossary: Some(53),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistWarp,
        talent_id: Some(24),
        icon_id: Some(img015::ICON_RESIST_WARP),
        cat_glossary: Some(54),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistCurse,
        talent_id: Some(30),
        icon_id: Some(img015::ICON_RESIST_CURSE),
        cat_glossary: Some(59),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistToxic,
        talent_id: Some(52),
        icon_id: Some(img015::ICON_RESIST_TOXIC),
        cat_glossary: Some(80),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::ResistSurge,
        talent_id: Some(54),
        icon_id: Some(img015::ICON_SURGE_RESIST),
        cat_glossary: Some(81),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[],
        attributes: |_| Vec::new(),
        apply_talent: Some(|_,_,_,_| {}),
    },
    Ability {
        identity: Identity::CostDown,
        talent_id: Some(25),
        icon_id: Some(img015::ICON_COST_DOWN),
        cat_glossary: Some(55),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Reduction", AttrUnit::None)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, reduction, _, _| stats.eoc1_cost = stats.eoc1_cost.saturating_sub(reduction)),
    },
    Ability {
        identity: Identity::RecoverSpeedUp,
        talent_id: Some(26),
        icon_id: Some(img015::ICON_RECOVER_SPEED_UP),
        cat_glossary: Some(56),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Reduction", AttrUnit::Frames)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, frames, _, _| stats.cooldown = stats.cooldown.saturating_sub(frames)),
    },
    Ability {
        identity: Identity::MoveSpeedUp,
        talent_id: Some(27),
        icon_id: Some(img015::ICON_MOVE_SPEED),
        cat_glossary: Some(57),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Increase", AttrUnit::None)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, speed, _, _| stats.speed += speed),
    },
    Ability {
        identity: Identity::AttackBuff,
        talent_id: Some(31),
        icon_id: Some(img015::ICON_ATTACK_BUFF),
        cat_glossary: Some(60),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Boost", AttrUnit::Percent)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.attack_1_damage = (stats.attack_1_damage as f32 * percentage_factor).round() as i32;
            stats.attack_2_damage = (stats.attack_2_damage as f32 * percentage_factor).round() as i32;
            stats.attack_3_damage = (stats.attack_3_damage as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: Identity::HealthBuff,
        talent_id: Some(32),
        icon_id: Some(img015::ICON_HEALTH_BUFF),
        cat_glossary: Some(61),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Boost", AttrUnit::Percent)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let percentage_factor = (100 + percent) as f32 / 100.0;
            stats.hitpoints = (stats.hitpoints as f32 * percentage_factor).round() as i32;
        }),
    },
    Ability {
        identity: Identity::TbaDown,
        talent_id: Some(61),
        icon_id: Some(img015::ICON_TBA_DOWN),
        cat_glossary: Some(87),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Reduction", AttrUnit::Percent)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, percent, _, _| {
            let time_reduction = (stats.attack_cooldown as f32 * percent as f32 / 100.0).round() as i32;
            stats.attack_cooldown = stats.attack_cooldown.saturating_sub(time_reduction);
        }),
    },
    Ability {
        identity: Identity::ImproveKnockbacks,
        talent_id: Some(28),
        icon_id: Some(img015::ICON_IMPROVE_KNOCKBACK_COUNT),
        cat_glossary: Some(58),
        enemy_glossary: None,
        orb_trait: None,
        schema: &[],
        talent_values: &[TalentValue::new("Increase", AttrUnit::None)],
        attributes: |_| Vec::new(),
        apply_talent: Some(|stats, count, _, _| stats.knockbacks += count),
    },
    Ability {
        identity: Identity::TimeBeforeDeath,
        talent_id: None,
        icon_id: None,
        cat_glossary: None,
        enemy_glossary: None,
        orb_trait: None,
        schema: &[("Duration", AttrUnit::Frames)],
        talent_values: &[],
        attributes: |stats| {
            if stats.time_before_death > -1 {
                vec![("Duration", AttrValue::Finite(stats.time_before_death), AttrUnit::Frames)]
            } else { Vec::new() }
        },
        apply_talent: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_talent_value_addresses_a_declared_pair() {
        for ability in REGISTRY {
            assert!(
                ability.talent_values.len() <= 4,
                "{:?} describes more pairs than a talent group declares",
                ability.identity,
            );
        }
    }

    #[test]
    fn only_appliable_talents_describe_pairs() {
        for ability in REGISTRY {
            assert!(
                ability.talent_values.is_empty() || ability.apply_talent.is_some(),
                "{:?} describes pairs it never consumes",
                ability.identity,
            );
        }
    }

    #[test]
    fn an_inverted_pair_round_trips_through_its_base() {
        assert_eq!(Stored::Direct.apply(30), 30);
        assert_eq!(Stored::Inverted(100).apply(30), 70);
        assert_eq!(Stored::Inverted(100).apply(Stored::Inverted(100).apply(30)), 30);
    }

    #[test]
    fn weaken_labels_the_pair_its_closure_reads_third() {
        let Some(weaken) = get_talent(1) else {
            panic!("talent 1 grants no ability");
        };

        let labels: Vec<&str> = weaken.talent_values.iter().map(|value| value.label).collect();
        assert_eq!(labels, ["Chance", "Duration", "Reduced To"]);

        let Some(reduced_to) = weaken.talent_values.get(2) else {
            panic!("weaken describes no third pair");
        };
        assert!(!reduced_to.interpolated);
        assert_eq!(reduced_to.stored, Stored::Inverted(100));

        let group = TalentGroup { min_3: 70, ..Default::default() };
        let mut stats = Entity::default();
        if let Some(apply) = weaken.apply_talent {
            apply(&mut stats, 10, 60, &group);
        }
        assert_eq!(stats.weaken_to, reduced_to.stored.apply(i32::from(group.min_3)));
    }

    #[test]
    fn a_surge_spawn_pair_is_quartered_and_never_interpolated() {
        let Some(surge) = get_talent(56) else {
            panic!("talent 56 grants no ability");
        };

        let group = TalentGroup { min_3: 400, max_3: 800, min_4: 200, max_4: 600, ..Default::default() };
        let mut stats = Entity::default();
        if let Some(apply) = surge.apply_talent {
            apply(&mut stats, 5, 1, &group);
        }

        let Some(anchor) = surge.talent_values.get(2) else {
            panic!("surge describes no third pair");
        };
        assert!(!anchor.interpolated);
        assert_eq!(anchor.scale, Scale::Quarter);
        assert_eq!(stats.surge_spawn_anchor, anchor.scale.apply(i32::from(group.min_3)));
    }
}
