use std::{
    cell,
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use crate::Fault;

use super::{
    AltarReward, AssetSource, BaseShake, BattleEffects, BattleEventLatch, BgEffects,
    BuiltDeckRecord, ButtonBank, CannonGrowthStep, CannonPart, CastleRow, CharaGroup, ComboStore,
    CounterSurgeEvent, DialogManager, DrawSink, DropRecord, EffectSprite, Enigma, EventItemStore,
    ExGroup, ExplosionEvent, FixedLineupStore, Imgcut, LabyrinthFloor, LineupRecord, Maanim,
    Mamodel, MapData, MapRecord, MetaHost, OrbStore, Platform, RankingRecord, ReleasePoint,
    RewardDef, SceneHost, ScoredMap, ScreenMetrics, SheetTable, SoundManager, SpecialRuleStore,
    StagePairRecord, StageRestriction, SurgeEvent, TextBlock, TextRenderer, Texture, TreasureStore,
    UiHost, UnlockGroup, WebPopupEntry,
};

pub const SIZE: usize = 0x500000;

pub const ENTITY_BASE: usize = 0x838f8;
pub const ENTITY_STRIDE: usize = 0x3e8;
pub const FACTION_STRIDE: usize = 0xc738;
pub const SLOTS_PER_FACTION: i32 = 51;

pub const STAGE_ENEMY_COLUMNS: usize = 14;

pub const UNIT_BUY: usize = 0x4af08;
pub const UNIT_BUY_STRIDE: usize = 0x100;

pub const CAT_STATS: usize = 0x9e568;
pub const CAT_STATS_UNIT_STRIDE: usize = 0x760;
pub const CAT_STATS_FORM_STRIDE: usize = 0x1d8;
pub const ENEMY_STATS: usize = 0x233108;
pub const ENEMY_STATS_STRIDE: usize = 0x1c4;

const FACTION_FLAGS: usize = 0x2648;
const FACTION_FLAGS_STRIDE: usize = 0x1f0;

const RNG_STATE: usize = 0x46f790;
const SITE: &str = "app_context";

const _: () = assert!(RNG_STATE + 4 <= SIZE);

pub struct UnitBuy;

impl UnitBuy {
    pub const RARITY: usize = 0x34;
    pub const GUIDE_ORDER: usize = 0x38;
    pub const TRUE_FORM_LEVEL: usize = 0x50;
    pub const MAX_LEVEL: usize = 0xc8;
    pub const MAX_PLUS_LEVEL: usize = 0xcc;
    pub const AVAILABLE: usize = 0xe4;
    pub const ALT_ART: usize = 0xf4;
    pub const KEY: usize = 0xfc;
}

pub struct PageList;

impl PageList {
    pub const MULTI_PAGE: usize = 0x3c;
    pub const ANIMATED: usize = 0x3d;
    pub const OFFSET: usize = 0x4c;
    pub const OFFSET_TARGET: usize = 0x50;
    pub const PAGE_COUNT: usize = 0x5c;
    pub const PAGE: usize = 0x60;
    pub const PAGE_TARGET: usize = 0x64;
    pub const PAGE_WIDTH: usize = 0x68;
}

pub struct Base;

impl Base {
    pub const OCCUPANT: usize = 0x0;
    pub const STATE: usize = 0x4;
    pub const CANNON_MAKES_WAVE: usize = 0x14;
    pub const LEVEL: usize = 0x18;
    pub const CASTLE_ANIM_STATE: usize = 0x24;
    pub const CASTLE_ANIM_FRAME: usize = 0x28;
    pub const CANNON_RECHARGE: usize = 0x2c;
    pub const CANNON_COUNTDOWN: usize = 0x30;
    pub const CANNON_SHOT_ID: usize = 0x34;
    pub const CANNON_BASE_DAMAGE: usize = 0x40;
    pub const CANNON_TYPE: usize = 0x44;
    pub const CANNON_STRIKE_X: usize = 0x48;
    pub const CANNON_DAMAGE: usize = 0x50;
    pub const CANNON_WALL_HP_PCT: usize = 0x54;
    pub const CANNON_STRIKE_WIDTH: usize = 0x54;
    pub const CANNON_WALL_LIFETIME: usize = 0x58;
    pub const CANNON_HP_MODE: usize = 0x58;
    pub const CANNON_ID: usize = 0x5c;
    pub const CANNON_FOUNDATION_ID: usize = 0x60;
    pub const CANNON_DECOR_ID: usize = 0x64;
    pub const CANNON_UNIT_ID: usize = 0x68;
    pub const CANNON_WALL_OFFSET: usize = 0x6c;
    pub const CANNON_METAL_PERMILLE: usize = 0x6c;
    pub const CANNON_ZOMBIE_PERMILLE: usize = 0x6c;
    pub const CANNON_RECOIL: usize = 0x70;
    pub const CANNON_READY_VFX: usize = 0x74;
    pub const CANNON_NONMETAL_PERMILLE: usize = 0x78;
    pub const CANNON_NONZOMBIE_PERMILLE: usize = 0x78;
    pub const CANNON_BURROWED_PERMILLE: usize = 0x7c;
}

pub struct ItemDefinition;

impl ItemDefinition {
    pub const KIND: usize = 0x0;
    pub const INDEX: usize = 0x4;
    pub const REDIRECT: usize = 0x8;
    pub const ICON: usize = 0x14;
}

pub struct WaveRecord;

impl WaveRecord {
    pub const KIND: usize = 0x0;
    pub const OWNER_SLOT: usize = 0x4;
    pub const IN_USE: usize = 0x8;
    pub const FRAME: usize = 0xc;
    pub const POS_X: usize = 0x10;
    pub const LEVEL: usize = 0x14;
    pub const ATTACK: usize = 0x18;
    pub const PROC_FLAGS: usize = 0x1c;
    pub const METAL_KILLER_PCT: usize = 0x28;
    pub const MINI: usize = 0x2c;
}

pub struct Pinch;

impl Pinch {
    pub const ACTIVE: usize = 0x0;
    pub const FIRST_DOWN: usize = 0x1;
    pub const SECOND_DOWN: usize = 0x2;
    pub const FIRST_X: usize = 0x4;
    pub const SECOND_X: usize = 0x8;
    pub const FIRST_Y: usize = 0xc;
    pub const SECOND_Y: usize = 0x10;
    pub const START: usize = 0x1c;
    pub const DISTANCE: usize = 0x2c;
    pub const PREV_DISTANCE: usize = 0x30;
}

pub struct WaveSprite;

impl WaveSprite {
    pub const STRIDE: usize = 0x8;
    pub const TIMER: usize = 0x0;
    pub const POS_X: usize = 0x4;
}

pub struct CannonShot;

impl CannonShot {
    pub const TIMER: usize = 0x0;
    pub const POS_X: usize = 0x4;
    pub const SHOT_ID: usize = 0x8;
}

pub struct VfxSlot;

impl VfxSlot {
    pub const ACTIVE: usize = 0x0;
    pub const POS_X: usize = 0x4;
    pub const POS_Y: usize = 0x8;
    pub const FRAME: usize = 0xc;
    pub const BROKEN: usize = 0x18;
}

pub struct Debris;

impl Debris {
    pub const TIMER: usize = 0x0;
    pub const POS_X: usize = 0x4;
    pub const POS_Y: usize = 0x8;
    pub const VARIANT: usize = 0xc;
}

pub struct Entity;

impl Entity {
    pub const OCCUPANT: usize = 0x0;
    pub const STATE: usize = 0x4;
    pub const FRAME: usize = 0x8;
    pub const POS_X: usize = 0xc;
    pub const POS_Y: usize = 0x10;
    pub const Z_LAYER: usize = 0x14;
    pub const LEVEL: usize = 0x18;
    pub const MAX_HP: usize = 0x1c;
    pub const HP: usize = 0x20;
    pub const KNOCKBACKS: usize = 0x24;
    pub const SPEED: usize = 0x28;
    pub const ATTACK_INTERVAL: usize = 0x30;
    pub const STANDING_RANGE: usize = 0x34;
    pub const HITBOX_POS: usize = 0x38;
    pub const HITBOX_WIDTH: usize = 0x3c;
    pub const TARGETS_RED: usize = 0x40;
    pub const IS_RED: usize = 0x44;
    pub const ATTACK_COOLDOWN: usize = 0x48;
    pub const FRAME_DAMAGE: usize = 0x4c;
    pub const AREA_ATTACK: usize = 0x50;
    pub const ATTACK_1_FORESWING: usize = 0x54;
    pub const CANNON_HIT_STAMP: usize = 0x58;
    pub const CANNON_BLAST_HIT: usize = 0x5c;
    pub const DEFEAT_FALL: usize = 0x60;
    pub const DEFEAT_ALPHA: usize = 0x64;
    pub const DEFEAT_DRIFT: usize = 0x68;
    pub const DEFEAT_SPIN: usize = 0x6c;
    pub const DEFEAT_BOUNCE: usize = 0x70;
    pub const DEFEAT_TIMER: usize = 0x74;
    pub const BACK_BOUND: usize = 0x78;
    pub const GOD_PUSH_FRAME: usize = 0x7c;
    pub const GOD_PUSH_STEP: usize = 0x80;
    pub const GOD_PUSH_DELAY: usize = 0x84;
    pub const CRIT_VFX: usize = 0x98;
    pub const WAVE_CHANCE: usize = 0x9c;
    pub const WAVE_LEVEL: usize = 0xa0;
    pub const TRAIT_FLOATING: usize = 0xa4;
    pub const TRAIT_DARK: usize = 0xa8;
    pub const TRAIT_METAL: usize = 0xac;
    pub const TRAIT_TRAITLESS: usize = 0xb0;
    pub const TRAIT_ANGEL: usize = 0xb4;
    pub const TRAIT_ALIEN: usize = 0xb8;
    pub const TRAIT_ZOMBIE: usize = 0xbc;
    pub const STRONG_AGAINST: usize = 0xc0;
    pub const KNOCKBACK_CHANCE: usize = 0xc4;
    pub const FREEZE_CHANCE: usize = 0xc8;
    pub const FREEZE_DURATION: usize = 0xcc;
    pub const SLOW_CHANCE: usize = 0xd0;
    pub const SLOW_DURATION: usize = 0xd4;
    pub const RESIST: usize = 0xd8;
    pub const MASSIVE_DAMAGE: usize = 0xdc;
    pub const CRITICAL_CHANCE: usize = 0xe0;
    pub const ATTACK_ONLY: usize = 0xe4;
    pub const DOUBLE_BOUNTY: usize = 0xe8;
    pub const BASE_DESTROYER: usize = 0xec;
    pub const KB_PROC_HIT: usize = 0xf0;
    pub const FREEZE_TIMER: usize = 0xf4;
    pub const SLOW_TIMER: usize = 0xf8;
    pub const DOUBLE_BOUNTY_STATE: usize = 0xfc;
    pub const BOSS_TYPE: usize = 0x100;
    pub const SHOCKWAVE_COUNTER: usize = 0x104;
    pub const FREEZE_LENGTH: usize = 0x108;
    pub const SLOW_LENGTH: usize = 0x10c;
    pub const WEAKEN_CHANCE: usize = 0x110;
    pub const WEAKEN_DURATION: usize = 0x114;
    pub const WEAKEN_PCT: usize = 0x118;
    pub const STRENGTHEN_THRESHOLD: usize = 0x11c;
    pub const STRENGTHEN_BOOST: usize = 0x120;
    pub const SURVIVE_CHANCE: usize = 0x124;
    pub const METAL_CAT: usize = 0x128;
    pub const WEAKEN_TIMER: usize = 0x12c;
    pub const WEAKEN_ACTIVE: usize = 0x130;
    pub const SURVIVE_USED: usize = 0x134;
    pub const SURVIVE_VFX_FRAME: usize = 0x138;
    pub const WEAKEN_ACTIVE_PCT: usize = 0x13c;
    pub const ATTACK_UP_VFX_FRAME: usize = 0x140;
    pub const ATTACK_1_LD_ANCHOR: usize = 0x150;
    pub const WAVE_IMMUNE: usize = 0x158;
    pub const WAVE_BLOCK: usize = 0x15c;
    pub const KNOCKBACK_IMMUNE: usize = 0x160;
    pub const FREEZE_IMMUNE: usize = 0x164;
    pub const SLOW_IMMUNE: usize = 0x168;
    pub const WEAKEN_IMMUNE: usize = 0x16c;
    pub const WAVE_IMMUNE_VFX_FRAME: usize = 0x170;
    pub const WAVE_IMMUNE_VFX_ACTIVE: usize = 0x174;
    pub const WAVE_BLOCK_VFX_FRAME: usize = 0x178;
    pub const WAVE_BLOCK_VFX_ACTIVE: usize = 0x17c;
    pub const HIT_SPARK_TYPE: usize = 0x180;
    pub const BURROW_COUNT: usize = 0x184;
    pub const BURROW_START_X: usize = 0x188;
    pub const IMMUNE_VFX_FRAME: usize = 0x194;
    pub const IMMUNE_VFX_ACTIVE: usize = 0x198;
    pub const REVIVE_COUNT: usize = 0x19c;
    pub const REVIVE_HP: usize = 0x1a0;
    pub const REVIVE_TIME: usize = 0x1a4;
    pub const BURROW_DISTANCE: usize = 0x1a8;
    pub const ZKILL_HIT: usize = 0x1ac;
    pub const ZOMBIE_KILLER: usize = 0x1b0;
    pub const TRAIT_WITCH: usize = 0x1b4;
    pub const WITCH_SLAYER: usize = 0x1b8;
    pub const TRAIT_DOJO: usize = 0x1bc;
    pub const TOTAL_DAMAGE_TAKEN: usize = 0x1c0;
    pub const SCORE_VALUE: usize = 0x1c4;
    pub const NO_REVIVE: usize = 0x1c8;
    pub const ATTACKS_REMAINING: usize = 0x1cc;
    pub const BOSS_WAVE_IMMUNE: usize = 0x1d0;
    pub const DEATH_TIMER: usize = 0x1d4;
    pub const ATTACK_END_MODE: usize = 0x1d8;
    pub const ATTACK_2_FORESWING: usize = 0x1e4;
    pub const ATTACK_3_FORESWING: usize = 0x1e8;
    pub const SPAWN_ANIM_TYPE: usize = 0x1f8;
    pub const SOUL_ANIM_TYPE: usize = 0x1fc;
    pub const SPAWN_ANIM_FLAG: usize = 0x200;
    pub const GUDETAMA_SOUL: usize = 0x204;
    pub const BARRIER_HP: usize = 0x208;
    pub const BARRIER_STATE: usize = 0x20c;
    pub const BARRIER_BREAKER_CHANCE: usize = 0x210;
    pub const BARRIER_VFX_ACTIVE: usize = 0x214;
    pub const BARRIER_VFX_FRAME: usize = 0x218;
    pub const WARP_CHANCE: usize = 0x21c;
    pub const WARP_DURATION: usize = 0x220;
    pub const WARP_ANCHOR: usize = 0x224;
    pub const WARP_SPAN: usize = 0x228;
    pub const WARP_TIMER: usize = 0x22c;
    pub const WARP_DISTANCE: usize = 0x230;
    pub const TRAIT_STARRED_ALIEN: usize = 0x238;
    pub const WARP_IMMUNE: usize = 0x23c;
    pub const TRAIT_EVA_ANGEL: usize = 0x240;
    pub const EVA_KILLER: usize = 0x244;
    pub const TRAIT_RELIC: usize = 0x248;
    pub const CURSE_CHANCE: usize = 0x24c;
    pub const CURSE_DURATION: usize = 0x250;
    pub const CURSE_IMMUNE: usize = 0x254;
    pub const CURSE_TIMER: usize = 0x258;
    pub const CURSE_LENGTH: usize = 0x25c;
    pub const WEAKEN_RESIST_PCT: usize = 0x264;
    pub const FREEZE_RESIST_PCT: usize = 0x268;
    pub const SLOW_RESIST_PCT: usize = 0x26c;
    pub const KNOCKBACK_RESIST_PCT: usize = 0x270;
    pub const WAVE_RESIST_PCT: usize = 0x274;
    pub const WARP_RESIST_PCT: usize = 0x278;
    pub const CURSE_RESIST_PCT: usize = 0x27c;
    pub const INSANELY_TOUGH: usize = 0x280;
    pub const INSANE_DAMAGE: usize = 0x284;
    pub const TOOK_DAMAGE: usize = 0x288;
    pub const SAVAGE_BLOW_CHANCE: usize = 0x28c;
    pub const SAVAGE_BLOW_BOOST: usize = 0x290;
    pub const DODGE_CHANCE: usize = 0x294;
    pub const SAVAGE_BLOW_VFX: usize = 0x298;
    pub const DODGE_TIMER: usize = 0x29c;
    pub const DODGE_DURATION: usize = 0x2a0;
    pub const DODGE_VFX_FRAME: usize = 0x2a4;
    pub const TOXIC_CHANCE: usize = 0x2ac;
    pub const TOXIC_DAMAGE: usize = 0x2b0;
    pub const TOXIC_VFX: usize = 0x2b4;
    pub const SURGE_CHANCE: usize = 0x2b8;
    pub const SURGE_ANCHOR: usize = 0x2bc;
    pub const SURGE_SPAN: usize = 0x2c0;
    pub const SURGE_LEVEL: usize = 0x2c4;
    pub const TOXIC_IMMUNE: usize = 0x2c8;
    pub const SURGE_IMMUNE: usize = 0x2cc;
    pub const TOXIC_RESIST_PCT: usize = 0x2d0;
    pub const SURGE_RESIST_PCT: usize = 0x2d4;
    pub const WAVE_MINI: usize = 0x2d8;
    pub const SHIELD_PIERCE_CHANCE: usize = 0x2dc;
    pub const SHIELD_MAX: usize = 0x2e0;
    pub const SHIELD_REGEN: usize = 0x2e4;
    pub const DEATH_SURGE_CHANCE: usize = 0x2e8;
    pub const DEATH_SURGE_ANCHOR: usize = 0x2ec;
    pub const DEATH_SURGE_SPAN: usize = 0x2f0;
    pub const DEATH_SURGE_LEVEL: usize = 0x2f4;
    pub const SHIELD_HP: usize = 0x2f8;
    pub const SHIELD_STATE: usize = 0x2fc;
    pub const SHIELD_VFX: usize = 0x300;
    pub const SHIELD_VFX_FRAME: usize = 0x304;
    pub const TRAIT_AKU: usize = 0x308;
    pub const HIT_FLASH_TIMER: usize = 0x30c;
    pub const TRAIT_COLOSSUS: usize = 0x310;
    pub const COLOSSUS_SLAYER: usize = 0x314;
    pub const SOULSTRIKE: usize = 0x318;
    pub const REVIVE_TIMER: usize = 0x334;
    pub const TRAIT_BEHEMOTH: usize = 0x338;
    pub const BEHEMOTH_SLAYER: usize = 0x33c;
    pub const BEHEMOTH_DODGE_CHANCE: usize = 0x340;
    pub const BEHEMOTH_DODGE_DURATION: usize = 0x344;
    pub const BEHEMOTH_DODGE_TIMER: usize = 0x348;
    pub const MINI_SURGE: usize = 0x34c;
    pub const COUNTER_SURGE: usize = 0x350;
    pub const CONJURE_UNIT_ID: usize = 0x354;
    pub const CONJURE_DECK_SLOT: usize = 0x358;
    pub const TRAIT_SAGE: usize = 0x35c;
    pub const SAGE_SLAYER: usize = 0x360;
    pub const SAGE_KB_RESIST_PCT: usize = 0x364;
    pub const METAL_KILLER_PCT: usize = 0x368;
    pub const METAL_KILLER_VFX: usize = 0x36c;
    pub const PREV_FREEZE_TIMER: usize = 0x370;
    pub const PREV_SLOW_TIMER: usize = 0x374;
    pub const PREV_WEAKEN_TIMER: usize = 0x378;
    pub const PREV_CURSE_TIMER: usize = 0x37c;
    pub const EXPLOSION_CHANCE: usize = 0x380;
    pub const EXPLOSION_ANCHOR: usize = 0x384;
    pub const EXPLOSION_SPAN: usize = 0x388;
    pub const EXPLOSION_IMMUNE: usize = 0x38c;
    pub const DEATH_SURGE_PCT: usize = 0x390;
    pub const COLOSSUS_ORB_ATK_PCT: usize = 0x394;
    pub const COLOSSUS_ORB_DEF_PCT: usize = 0x398;
    pub const CASH_BACK_PCT: usize = 0x39c;
    pub const SCORE_HIT_MASK: usize = 0x3a0;
    pub const DEATH_SURGE_MINI: usize = 0x3a4;
    pub const ORB_DODGE_TIMER: usize = 0x3a8;
    pub const ORB_DODGE_CHANCE: usize = 0x3ac;
    pub const ORB_DODGE_DURATION: usize = 0x3b0;
    pub const CANNON_CHARGE_ORB: usize = 0x3b4;
    pub const COUNTER_SURGE_PCT: usize = 0x3b8;
    pub const COUNTER_SURGE_ONCE: usize = 0x3bc;
    pub const SPAWN_SERIAL: usize = 0x3c0;
    pub const KILL_COUNT: usize = 0x3c4;
    pub const PAID_COST: usize = 0x3c8;
    pub const FIRST_BOUNTY_PCT: usize = 0x3cc;
    pub const TRAIT_KAIJIN: usize = 0x3d0;
    pub const EXPLOSION_RESIST_PCT: usize = 0x3d4;
    pub const DRAIN_CHANCE: usize = 0x3d8;
    pub const DRAIN_PERCENT: usize = 0x3dc;
    pub const DRAIN_PCT: usize = 0x3e0;
    pub const DRAIN_IMMUNE: usize = 0x3e4;
}

pub struct CatStats;

impl CatStats {
    pub const HITPOINTS: usize = 0x0;
    pub const KNOCKBACKS: usize = 0x4;
    pub const SPEED: usize = 0x8;
    pub const ATTACK_1_DAMAGE: usize = 0xc;
    pub const ATTACK_COOLDOWN: usize = 0x10;
    pub const STANDING_RANGE: usize = 0x14;
    pub const EOC1_COST: usize = 0x18;
    pub const COOLDOWN: usize = 0x1c;
    pub const HITBOX_POSITION: usize = 0x20;
    pub const HITBOX_WIDTH: usize = 0x24;
    pub const TARGET_RED: usize = 0x28;
    pub const LEGACY_WEAK_AGAINST: usize = 0x2c;
    pub const AREA_ATTACK: usize = 0x30;
    pub const TIME_UNTIL_ATTACK_1: usize = 0x34;
    pub const MINIMUM_Z_LAYER: usize = 0x38;
    pub const MAXIMUM_Z_LAYER: usize = 0x3c;
    pub const TARGET_FLOATING: usize = 0x40;
    pub const TARGET_DARK: usize = 0x44;
    pub const TARGET_METAL: usize = 0x48;
    pub const TARGET_TRAITLESS: usize = 0x4c;
    pub const TARGET_ANGEL: usize = 0x50;
    pub const TARGET_ALIEN: usize = 0x54;
    pub const TARGET_ZOMBIE: usize = 0x58;
    pub const STRONG_AGAINST: usize = 0x5c;
    pub const KNOCKBACK_CHANCE: usize = 0x60;
    pub const FREEZE_CHANCE: usize = 0x64;
    pub const FREEZE_DURATION: usize = 0x68;
    pub const SLOW_CHANCE: usize = 0x6c;
    pub const SLOW_DURATION: usize = 0x70;
    pub const RESIST: usize = 0x74;
    pub const MASSIVE_DAMAGE: usize = 0x78;
    pub const CRITICAL_CHANCE: usize = 0x7c;
    pub const ATTACK_ONLY: usize = 0x80;
    pub const DOUBLE_BOUNTY: usize = 0x84;
    pub const BASE_DESTROYER: usize = 0x88;
    pub const WAVE_CHANCE: usize = 0x8c;
    pub const WAVE_LEVEL: usize = 0x90;
    pub const WEAKEN_CHANCE: usize = 0x94;
    pub const WEAKEN_DURATION: usize = 0x98;
    pub const WEAKEN_TO: usize = 0x9c;
    pub const STRENGTHEN_THRESHOLD: usize = 0xa0;
    pub const STRENGTHEN_BOOST: usize = 0xa4;
    pub const SURVIVE: usize = 0xa8;
    pub const IS_METAL: usize = 0xac;
    pub const LD1_ANCHOR: usize = 0xb0;
    pub const LD1_SPAN: usize = 0xb4;
    pub const WAVE_IMMUNE: usize = 0xb8;
    pub const WAVE_BLOCK: usize = 0xbc;
    pub const KNOCKBACK_IMMUNE: usize = 0xc0;
    pub const FREEZE_IMMUNE: usize = 0xc4;
    pub const SLOW_IMMUNE: usize = 0xc8;
    pub const WEAKEN_IMMUNE: usize = 0xcc;
    pub const ZOMBIE_KILLER: usize = 0xd0;
    pub const WITCH_KILLER: usize = 0xd4;
    pub const TARGET_WITCH: usize = 0xd8;
    pub const ATTACK_COUNT_TOTAL: usize = 0xdc;
    pub const BOSS_WAVE_IMMUNE: usize = 0xe0;
    pub const TIME_BEFORE_DEATH: usize = 0xe4;
    pub const ATTACK_COUNT_STATE: usize = 0xe8;
    pub const ATTACK_2_DAMAGE: usize = 0xec;
    pub const ATTACK_3_DAMAGE: usize = 0xf0;
    pub const TIME_UNTIL_ATTACK_2: usize = 0xf4;
    pub const TIME_UNTIL_ATTACK_3: usize = 0xf8;
    pub const ATTACK_1_ABILITIES: usize = 0xfc;
    pub const ATTACK_2_ABILITIES: usize = 0x100;
    pub const ATTACK_3_ABILITIES: usize = 0x104;
    pub const SPAWN_ANIMATION_TYPE: usize = 0x108;
    pub const SOUL_ANIMATION_TYPE: usize = 0x10c;
    pub const SPAWN_ANIMATION_FLAG: usize = 0x110;
    pub const USE_GUDETAMA_SOUL: usize = 0x114;
    pub const BARRIER_BREAKER_CHANCE: usize = 0x118;
    pub const WARP_CHANCE: usize = 0x11c;
    pub const WARP_DURATION: usize = 0x120;
    pub const WARP_DIST_ANCHOR: usize = 0x124;
    pub const WARP_DIST_SPAN: usize = 0x128;
    pub const WARP_IMMUNE: usize = 0x12c;
    pub const TARGET_EVA: usize = 0x130;
    pub const EVA_KILLER: usize = 0x134;
    pub const TARGET_RELIC: usize = 0x138;
    pub const CURSE_IMMUNE: usize = 0x13c;
    pub const INSANELY_TOUGH: usize = 0x140;
    pub const INSANE_DAMAGE: usize = 0x144;
    pub const SAVAGE_BLOW_CHANCE: usize = 0x148;
    pub const SAVAGE_BLOW_BOOST: usize = 0x14c;
    pub const DODGE_CHANCE: usize = 0x150;
    pub const DODGE_DURATION: usize = 0x154;
    pub const SURGE_CHANCE: usize = 0x158;
    pub const SURGE_SPAWN_ANCHOR: usize = 0x15c;
    pub const SURGE_SPAWN_SPAN: usize = 0x160;
    pub const SURGE_LEVEL: usize = 0x164;
    pub const TOXIC_IMMUNE: usize = 0x168;
    pub const SURGE_IMMUNE: usize = 0x16c;
    pub const CURSE_CHANCE: usize = 0x170;
    pub const CURSE_DURATION: usize = 0x174;
    pub const MINI_WAVE_FLAG: usize = 0x178;
    pub const SHIELD_PIERCE_CHANCE: usize = 0x17c;
    pub const TARGET_AKU: usize = 0x180;
    pub const COLOSSUS_SLAYER: usize = 0x184;
    pub const SOULSTRIKE: usize = 0x188;
    pub const LD2_FLAG: usize = 0x18c;
    pub const LD2_ANCHOR: usize = 0x190;
    pub const LD2_SPAN: usize = 0x194;
    pub const LD3_FLAG: usize = 0x198;
    pub const LD3_ANCHOR: usize = 0x19c;
    pub const LD3_SPAN: usize = 0x1a0;
    pub const BEHEMOTH_SLAYER: usize = 0x1a4;
    pub const BEHEMOTH_DODGE_CHANCE: usize = 0x1a8;
    pub const BEHEMOTH_DODGE_DURATION: usize = 0x1ac;
    pub const MINI_SURGE_FLAG: usize = 0x1b0;
    pub const COUNTER_SURGE: usize = 0x1b4;
    pub const CONJURE_UNIT_ID: usize = 0x1b8;
    pub const SAGE_SLAYER: usize = 0x1bc;
    pub const METAL_KILLER_PERCENT: usize = 0x1c0;
    pub const EXPLOSION_CHANCE: usize = 0x1c4;
    pub const EXPLOSION_SPAWN_ANCHOR: usize = 0x1c8;
    pub const EXPLOSION_SPAWN_SPAN: usize = 0x1cc;
    pub const EXPLOSION_IMMUNE: usize = 0x1d0;
    pub const DRAIN_IMMUNE: usize = 0x1d4;
}

pub struct EnemyStats;

impl EnemyStats {
    pub const HITPOINTS: usize = 0x0;
    pub const KNOCKBACKS: usize = 0x4;
    pub const SPEED: usize = 0x8;
    pub const ATTACK_1_DAMAGE: usize = 0xc;
    pub const ATTACK_COOLDOWN: usize = 0x10;
    pub const STANDING_RANGE: usize = 0x14;
    pub const CASH_DROP: usize = 0x18;
    pub const HITBOX_POSITION: usize = 0x1c;
    pub const HITBOX_WIDTH: usize = 0x20;
    pub const LEGACY_STRONG_AGAINST: usize = 0x24;
    pub const TRAIT_RED: usize = 0x28;
    pub const AREA_ATTACK: usize = 0x2c;
    pub const TIME_UNTIL_ATTACK_1: usize = 0x30;
    pub const TRAIT_FLOATING: usize = 0x34;
    pub const TRAIT_DARK: usize = 0x38;
    pub const TRAIT_METAL: usize = 0x3c;
    pub const TRAIT_TRAITLESS: usize = 0x40;
    pub const TRAIT_ANGEL: usize = 0x44;
    pub const TRAIT_ALIEN: usize = 0x48;
    pub const TRAIT_ZOMBIE: usize = 0x4c;
    pub const KNOCKBACK_CHANCE: usize = 0x50;
    pub const FREEZE_CHANCE: usize = 0x54;
    pub const FREEZE_DURATION: usize = 0x58;
    pub const SLOW_CHANCE: usize = 0x5c;
    pub const SLOW_DURATION: usize = 0x60;
    pub const CRITICAL_CHANCE: usize = 0x64;
    pub const BASE_DESTROYER: usize = 0x68;
    pub const WAVE_CHANCE: usize = 0x6c;
    pub const WAVE_LEVEL: usize = 0x70;
    pub const WEAKEN_CHANCE: usize = 0x74;
    pub const WEAKEN_DURATION: usize = 0x78;
    pub const WEAKEN_TO: usize = 0x7c;
    pub const STRENGTHEN_THRESHOLD: usize = 0x80;
    pub const STRENGTHEN_BOOST: usize = 0x84;
    pub const SURVIVE: usize = 0x88;
    pub const LD1_ANCHOR: usize = 0x8c;
    pub const LD1_SPAN: usize = 0x90;
    pub const WAVE_IMMUNE: usize = 0x94;
    pub const WAVE_BLOCK: usize = 0x98;
    pub const KNOCKBACK_IMMUNE: usize = 0x9c;
    pub const FREEZE_IMMUNE: usize = 0xa0;
    pub const SLOW_IMMUNE: usize = 0xa4;
    pub const WEAKEN_IMMUNE: usize = 0xa8;
    pub const BURROW_AMOUNT: usize = 0xac;
    pub const BURROW_DISTANCE: usize = 0xb0;
    pub const REVIVE_COUNT: usize = 0xb4;
    pub const REVIVE_TIME: usize = 0xb8;
    pub const REVIVE_HP: usize = 0xbc;
    pub const TRAIT_WITCH: usize = 0xc0;
    pub const TRAIT_DOJO: usize = 0xc4;
    pub const ATTACK_COUNT_TOTAL: usize = 0xc8;
    pub const TIME_BEFORE_DEATH: usize = 0xcc;
    pub const ATTACK_COUNT_STATE: usize = 0xd0;
    pub const SPAWN_ANIMATION_TYPE: usize = 0xd4;
    pub const SOUL_ANIMATION_TYPE: usize = 0xd8;
    pub const ATTACK_2_DAMAGE: usize = 0xdc;
    pub const ATTACK_3_DAMAGE: usize = 0xe0;
    pub const TIME_UNTIL_ATTACK_2: usize = 0xe4;
    pub const TIME_UNTIL_ATTACK_3: usize = 0xe8;
    pub const ATTACK_1_ABILITIES: usize = 0xec;
    pub const ATTACK_2_ABILITIES: usize = 0xf0;
    pub const ATTACK_3_ABILITIES: usize = 0xf4;
    pub const SPAWN_ANIMATION_FLAG: usize = 0xf8;
    pub const USE_GUDETAMA_SOUL: usize = 0xfc;
    pub const BARRIER_HITPOINTS: usize = 0x100;
    pub const WARP_CHANCE: usize = 0x104;
    pub const WARP_DURATION: usize = 0x108;
    pub const WARP_DIST_ANCHOR: usize = 0x10c;
    pub const WARP_DIST_SPAN: usize = 0x110;
    pub const TRAIT_STARRED_ALIEN: usize = 0x114;
    pub const WARP_IMMUNE: usize = 0x118;
    pub const TRAIT_EVA: usize = 0x11c;
    pub const TRAIT_RELIC: usize = 0x120;
    pub const CURSE_CHANCE: usize = 0x124;
    pub const CURSE_DURATION: usize = 0x128;
    pub const SAVAGE_BLOW_CHANCE: usize = 0x12c;
    pub const SAVAGE_BLOW_BOOST: usize = 0x130;
    pub const DODGE_CHANCE: usize = 0x134;
    pub const DODGE_DURATION: usize = 0x138;
    pub const TOXIC_CHANCE: usize = 0x13c;
    pub const TOXIC_DAMAGE: usize = 0x140;
    pub const SURGE_CHANCE: usize = 0x144;
    pub const SURGE_SPAWN_ANCHOR: usize = 0x148;
    pub const SURGE_SPAWN_SPAN: usize = 0x14c;
    pub const SURGE_LEVEL: usize = 0x150;
    pub const SURGE_IMMUNE: usize = 0x154;
    pub const MINI_WAVE_FLAG: usize = 0x158;
    pub const SHIELD_HITPOINTS: usize = 0x15c;
    pub const SHIELD_REGEN: usize = 0x160;
    pub const DEATH_SURGE_CHANCE: usize = 0x164;
    pub const DEATH_SURGE_ANCHOR: usize = 0x168;
    pub const DEATH_SURGE_SPAN: usize = 0x16c;
    pub const DEATH_SURGE_LEVEL: usize = 0x170;
    pub const TRAIT_AKU: usize = 0x174;
    pub const TRAIT_COLOSSUS: usize = 0x178;
    pub const LD2_FLAG: usize = 0x17c;
    pub const LD2_ANCHOR: usize = 0x180;
    pub const LD2_SPAN: usize = 0x184;
    pub const LD3_FLAG: usize = 0x188;
    pub const LD3_ANCHOR: usize = 0x18c;
    pub const LD3_SPAN: usize = 0x190;
    pub const TRAIT_BEHEMOTH: usize = 0x194;
    pub const MINI_SURGE_FLAG: usize = 0x198;
    pub const COUNTER_SURGE: usize = 0x19c;
    pub const TRAIT_SAGE: usize = 0x1a0;
    pub const CURSE_IMMUNE: usize = 0x1a4;
    pub const EXPLOSION_CHANCE: usize = 0x1a8;
    pub const EXPLOSION_SPAWN_ANCHOR: usize = 0x1ac;
    pub const EXPLOSION_SPAWN_SPAN: usize = 0x1b0;
    pub const EXPLOSION_IMMUNE: usize = 0x1b4;
    pub const TRAIT_KAIJIN: usize = 0x1b8;
    pub const DRAIN_CHANCE: usize = 0x1bc;
    pub const DRAIN_PERCENT: usize = 0x1c0;
}

pub struct AppContext {
    raw: Box<[u8]>,
    pub stage_enemies: Vec<[i32; STAGE_ENEMY_COLUMNS]>,
    pub spawn_states: Vec<[i32; 3]>,
    pub unit_models: [Vec<Mamodel>; 2],
    pub unit_anims: [Vec<BTreeMap<i32, Maanim>>; 2],
    pub death_surge_anims: [Maanim; 2],
    pub counter_surge_anims: [Maanim; 2],
    pub wave_anim: Maanim,
    pub mini_wave_anim: Maanim,
    pub effect_anims: BTreeMap<i32, Maanim>,
    pub base_anims: [Maanim; 4],
    pub battle_event_latch: BattleEventLatch,
    pub event_items: Option<EventItemStore>,
    pub listed_item_counts: Vec<[i32; 2]>,
    pub metal_killer_map: BTreeMap<i32, Vec<i32>>,
    pub surge_events: Vec<SurgeEvent>,
    pub counter_surge_events: Vec<CounterSurgeEvent>,
    pub explosion_events: Vec<ExplosionEvent>,
    pub base_shake: BaseShake,
    pub attackers_by_serial: [BTreeMap<i32, Vec<i32>>; 2],
    pub scored_maps: BTreeMap<i32, ScoredMap>,
    pub cleared_session_keys: Vec<i32>,
    pub conditioned_maps: Vec<i32>,
    pub stage_conditions: BTreeMap<i32, BTreeMap<i32, BTreeMap<i32, i32>>>,
    pub legend_stage_conditions: Vec<[i32; 9]>,
    pub aku_stage_lists: BTreeMap<i32, Vec<i32>>,
    pub unlock_groups: BTreeMap<i32, UnlockGroup>,
    pub unlock_flags: BTreeMap<i32, bool>,
    pub condition_list_200k: Vec<i32>,
    pub condition_list_300k: Vec<i32>,
    pub deploy_queue: Vec<u64>,
    pub altar_level_caps: BTreeMap<i32, i32>,
    pub altar_unsealed: BTreeMap<i32, bool>,
    pub screen_metrics: ScreenMetrics,
    pub deck_bar_base_y: i32,
    pub stage_restrictions: BTreeMap<i32, StageRestriction>,
    pub cannon_type_names: BTreeMap<i32, Vec<u8>>,
    pub enemy_names: Vec<Vec<u8>>,
    pub cat_names: Vec<[[Vec<u8>; 5]; 4]>,
    pub item_names: Vec<Vec<u8>>,
    pub item_descriptions: Vec<[Vec<u8>; 3]>,
    pub unit_info_texts: [Option<Texture>; 4],
    pub localizable: BTreeMap<Vec<u8>, Vec<u8>>,
    pub combo_names: Vec<Vec<u8>>,
    pub combo_effect_texts: Vec<Vec<u8>>,
    pub combo_power_texts: Vec<Vec<u8>>,
    pub map_records: BTreeMap<i32, MapRecord>,
    pub bg_effects: BgEffects,
    pub bg_anim_cache: BTreeMap<Vec<u8>, Maanim>,
    pub dialogs: DialogManager,
    pub buttons: ButtonBank,
    pub unlock_popups: BTreeMap<i32, bool>,
    pub img001_sheet: Option<Rc<Imgcut>>,
    pub outro_event_sheets: [Option<Rc<Imgcut>>; 3],
    pub texture_cache: BTreeMap<Vec<u8>, Weak<Imgcut>>,
    pub reward_queue: Vec<Vec<i32>>,
    pub item_possession: BTreeMap<i32, i32>,
    pub treasure_names: Vec<Vec<u8>>,
    pub stage_names: Vec<Vec<Vec<u8>>>,
    pub next_stage_names: Vec<Vec<[Vec<u8>; 3]>>,
    pub next_stage_caption: Vec<u8>,
    pub ex_stage_names: Vec<Vec<Vec<u8>>>,
    pub ex_groups: BTreeMap<i32, ExGroup>,
    pub ex_lottery: Vec<[i32; 2]>,
    pub hidden_drop_keys: BTreeMap<i32, Vec<u8>>,
    pub stage_pair_records: BTreeMap<i32, StagePairRecord>,
    pub altar_enemy_ids: BTreeMap<i32, i32>,
    pub altar_rewards: BTreeMap<i32, AltarReward>,
    pub label_texts: Vec<Option<Texture>>,
    pub warning2_rows: Vec<[Vec<u8>; 5]>,
    pub lose_rows: Vec<Vec<Vec<u8>>>,
    pub lose_row_settings: Vec<i32>,
    pub lose_text_settings: Vec<Vec<i32>>,
    pub web_popup_pending: BTreeMap<i32, bool>,
    pub img004_sheet: Option<Rc<Imgcut>>,
    pub reward_defs: BTreeMap<i32, Vec<RewardDef>>,
    pub reward_claimed: BTreeMap<i32, Vec<i32>>,
    pub release_points: BTreeMap<i32, ReleasePoint>,
    pub server_flags: BTreeMap<i32, bool>,
    pub event_unit_rows: Vec<Vec<i32>>,
    pub drop_chara_max_1000: i32,
    pub drop_chara_max_1100: i32,
    pub img039_sheet: Option<Rc<Imgcut>>,
    pub deck_button_x: [i32; 10],
    pub tooltip_texts: [Option<Texture>; 16],
    pub enemy_kill_counts: BTreeMap<i32, i32>,
    pub best_scores: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub ranking_entries: Vec<Option<RankingRecord>>,
    pub battle_effects: BattleEffects,
    pub boss_shockwave_anim: Maanim,
    pub cannon_part_rows: BTreeMap<i32, Vec<i32>>,
    pub maps_neg26: Vec<[u64; 3]>,
    pub maps_neg24: Vec<[u64; 3]>,
    pub maps_neg23: Vec<[u64; 3]>,
    pub maps_neg22: Vec<[u64; 3]>,
    pub maps_neg21: Vec<[u64; 3]>,
    pub maps_neg20: Vec<[u64; 3]>,
    pub maps_neg19: Vec<[u64; 3]>,
    pub maps_neg18: Vec<[u64; 3]>,
    pub maps_neg17: Vec<[u64; 3]>,
    pub maps_neg16: Vec<[u64; 3]>,
    pub maps_neg11: Vec<[u64; 3]>,
    pub maps_neg10: Vec<[u64; 3]>,
    pub maps_neg9: Vec<[u64; 3]>,
    pub maps_neg4: Vec<[u64; 3]>,
    pub ex_option_targets: BTreeMap<i32, i32>,
    pub ex_replacement_stages: BTreeMap<i32, Vec<i32>>,
    pub built_deck_stages: BTreeMap<i16, Vec<i32>>,
    pub built_deck_records: BTreeMap<i16, BuiltDeckRecord>,
    pub cannon_parts: BTreeMap<i32, CannonPart>,
    pub enemy_castle: Vec<CastleRow>,
    pub fixed_lineup_store: FixedLineupStore,
    pub combo_store: ComboStore,
    pub chara_groups: BTreeMap<i32, CharaGroup>,
    pub star_multipliers: BTreeMap<i32, Vec<i32>>,
    pub map_cost_multipliers: BTreeMap<i32, i32>,
    pub settings: BTreeMap<Vec<u8>, Vec<u8>>,
    pub treasure_store: TreasureStore,
    pub orb_store: OrbStore,
    pub special_rules: SpecialRuleStore,
    pub equipped_orbs: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub map_data: BTreeMap<i32, MapData>,
    pub map_data_ids: BTreeMap<i32, Vec<i32>>,
    pub map_stage_sets: BTreeMap<i32, Vec<i32>>,
    pub talent_definitions: BTreeMap<i32, [i32; 0x71]>,
    pub talent_levels: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub outbreak_active: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub outbreak_cleared: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub cleared_map_ids: Vec<i32>,
    pub stages_cleared_cache: BTreeMap<i32, [i16; 4]>,
    pub stages_cleared_neg24: Vec<Vec<i8>>,
    pub stages_cleared_neg23: Vec<Vec<i8>>,
    pub stages_cleared_neg22: Vec<Vec<i8>>,
    pub stages_cleared_neg20: Vec<i8>,
    pub stages_cleared_neg18: Vec<i8>,
    pub stages_cleared_neg17: Vec<i8>,
    pub stages_cleared_neg16: Vec<i8>,
    pub stages_cleared_neg11: Vec<i8>,
    pub stages_cleared_neg10: Vec<i32>,
    pub stages_cleared_neg9: Vec<i32>,
    pub stages_cleared_neg4: Vec<i32>,
    pub stage_record_cache: BTreeMap<i32, BTreeMap<i32, [i16; 4]>>,
    pub download_sheet: Option<Rc<Imgcut>>,
    pub deploy_cost_alt_sheet: Option<Rc<Imgcut>>,
    pub scene_img005_sheet: Option<Rc<Imgcut>>,
    pub trait_icons: BTreeMap<i32, bool>,
    pub ability_icons: BTreeMap<i32, bool>,
    pub picture_book_abilities: Vec<Vec<i32>>,
    pub picture_book_traits: Vec<Vec<i32>>,
    pub picture_book_trait_order: Vec<i32>,
    pub unit_icon_textures: [Option<Rc<Imgcut>>; 10],
    pub enemy_sheets: SheetTable,
    pub unit_sheets: [SheetTable; 4],
    pub enemy_castle_sheet: Option<Rc<Imgcut>>,
    pub base_models: [Mamodel; 4],
    pub base_sheets: SheetTable,
    pub bg_effect_sheets: BTreeMap<Vec<u8>, Option<Rc<Imgcut>>>,
    pub option_window_sheet: Option<Rc<Imgcut>>,
    pub effect_models: BTreeMap<i32, Mamodel>,
    pub effect_sheets: BTreeMap<i32, Option<Rc<Imgcut>>>,
    pub smallvolcano_anims: [Maanim; 3],
    pub volcano_anims: [Maanim; 3],
    pub warp_anims: [Maanim; 4],
    pub explosion_anims: [Maanim; 2],
    pub castle_anims: [Maanim; 8],
    pub castle_models: [Mamodel; 8],
    pub skill_effect_invalid_anim: Maanim,
    pub skill_wave_stop_e_anim: Maanim,
    pub skill_wave_invalid_e_anim: Maanim,
    pub skill_down_e_anim: Maanim,
    pub skill_shield_e_anim: Maanim,
    pub skill_stop_e_anim: Maanim,
    pub skill_slow_e_anim: Maanim,
    pub skill_up_e_anim: Maanim,
    pub skill_wave_stop_anim: Maanim,
    pub skill_wave_invalid_anim: Maanim,
    pub skill_down_anim: Maanim,
    pub skill_shield_anim: Maanim,
    pub skill_stop_anim: Maanim,
    pub skill_slow_anim: Maanim,
    pub skill_up_anim: Maanim,
    pub invoke_equipment_anim: Maanim,
    pub fever_anim: Maanim,
    pub attack_invalid_anim: Maanim,
    pub guard_e_breaker_anim: Maanim,
    pub guard_e_anim: Maanim,
    pub smallwave_attack_e_anim: Maanim,
    pub wave_attack_e_anim: Maanim,
    pub skill_curse_e_anim: Maanim,
    pub skill_curse_anim: Maanim,
    pub zombie_back_anim: Maanim,
    pub zombie_revive_anim: Maanim,
    pub zombie_down_anim: Maanim,
    pub skill_effect_invalid_model: Mamodel,
    pub skill_wave_stop_e_model: Mamodel,
    pub skill_wave_invalid_e_model: Mamodel,
    pub skill_down_e_model: Mamodel,
    pub skill_shield_e_model: Mamodel,
    pub skill_stop_e_model: Mamodel,
    pub skill_slow_e_model: Mamodel,
    pub skill_up_e_model: Mamodel,
    pub skill_wave_stop_model: Mamodel,
    pub skill_wave_invalid_model: Mamodel,
    pub skill_down_model: Mamodel,
    pub skill_shield_model: Mamodel,
    pub skill_stop_model: Mamodel,
    pub skill_slow_model: Mamodel,
    pub skill_up_model: Mamodel,
    pub demonbattle_model: Mamodel,
    pub invoke_equipment_model: Mamodel,
    pub fever_model: Mamodel,
    pub recast_decrease_e_model: Mamodel,
    pub metal_strong_model: Mamodel,
    pub smallvolcano_e_model: Mamodel,
    pub smallvolcano_model: Mamodel,
    pub volcano_e_model: Mamodel,
    pub volcano_model: Mamodel,
    pub percentage_attack_model: Mamodel,
    pub attack_invalid_model: Mamodel,
    pub strong_attack_model: Mamodel,
    pub warp_chara_model: Mamodel,
    pub warp_model: Mamodel,
    pub demonsummon_e_model: Mamodel,
    pub demonsummon_model: Mamodel,
    pub demonsoul_01_model: Mamodel,
    pub demonsoul_00_model: Mamodel,
    pub guard_e_model: Mamodel,
    pub demonshield_model: Mamodel,
    pub barrier_model: Mamodel,
    pub explosion_e_model: Mamodel,
    pub explosion_model: Mamodel,
    pub smallwave_attack_e_model: Mamodel,
    pub smallwave_attack_model: Mamodel,
    pub wave_attack_e_model: Mamodel,
    pub wave_attack_model: Mamodel,
    pub skill_curse_e_model: Mamodel,
    pub skill_curse_model: Mamodel,
    pub skill_zombie_strong_model: Mamodel,
    pub zombie_model: Mamodel,
    pub crit_vfx_model: Mamodel,
    pub boss_welcome_model: Mamodel,
    pub sealed_announce_sheets: [Option<Rc<Imgcut>>; 3],
    pub demonsoul_sheets: [Option<Rc<Imgcut>>; 2],
    pub skill_sheets: SheetTable,
    pub zombie_sheets: [Option<Rc<Imgcut>>; 2],
    pub equipment_effect_s_sheet: Option<Rc<Imgcut>>,
    pub equipment_attribute_s_sheet: Option<Rc<Imgcut>>,
    pub equipment_shadow_sheet: Option<Rc<Imgcut>>,
    pub equipment_grade_sheet: Option<Rc<Imgcut>>,
    pub equipment_effect_sheet: Option<Rc<Imgcut>>,
    pub equipment_attribute_sheet: Option<Rc<Imgcut>>,
    pub img015_sheet: Option<Rc<Imgcut>>,
    pub fever_sheet: Option<Rc<Imgcut>>,
    pub effect_a_sheet: Option<Rc<Imgcut>>,
    pub castle_sheet: Option<Rc<Imgcut>>,
    pub bubble_sheet: Option<Rc<Imgcut>>,
    pub img001_second_sheet: Option<Rc<Imgcut>>,
    pub img101_sheet: Option<Rc<Imgcut>>,
    pub img100_sheet: Option<Rc<Imgcut>>,
    pub img042_sheet: Option<Rc<Imgcut>>,
    pub img041_sheet: Option<Rc<Imgcut>>,
    pub img040_sheet: Option<Rc<Imgcut>>,
    pub img006_sheet: Option<Rc<Imgcut>>,
    pub img043_sheet: Option<Rc<Imgcut>>,
    pub stage_name_sheet: Option<Rc<Imgcut>>,
    pub img060_sheet: Option<Rc<Imgcut>>,
    pub img024_sheet: Option<Rc<Imgcut>>,
    pub bg_sheet: Option<Rc<Imgcut>>,
    pub combo_definitions: Vec<Vec<i32>>,
    pub play_dungeon_rows: Vec<[i32; 9]>,
    pub restriction_warning_texts: [Option<Texture>; 3],
    pub bg_anim_names: BTreeMap<i32, Vec<u8>>,
    pub bg_model_names: BTreeMap<i32, Vec<u8>>,
    pub bg_models: BTreeMap<Vec<u8>, Mamodel>,
    pub castle_hp_growth: Vec<CannonGrowthStep>,
    pub dungeon_clear_counts: Vec<[[i16; 4]; 0x30]>,
    pub random_dungeon_rows: Vec<[i32; 11]>,
    pub entry_record_maps: Vec<i32>,
    pub data_pack_expected: Vec<u8>,
    pub data_pack_key: Vec<u8>,
    pub map_stage_options: BTreeMap<i32, Vec<[i32; 4]>>,
    pub ranking_maps: BTreeMap<i32, i32>,
    pub event_reward_maps: BTreeMap<i32, i32>,
    pub text_blocks: BTreeMap<i32, TextBlock>,
    pub mapicon_sheet: Option<Rc<Imgcut>>,
    pub img003_sheet: Option<Rc<Imgcut>>,
    pub img002_sheet: Option<Rc<Imgcut>>,
    pub draw: Option<Box<dyn DrawSink>>,
    pub miracle_anims: [[Maanim; 2]; 4],
    pub miracle_levels: [[u8; 8]; 4],
    pub battle_option_texts: Vec<Vec<u8>>,
    pub battle_menu_texts: Vec<Vec<u8>>,
    pub battle_texts: Vec<Vec<u8>>,
    pub god_item_texts: Vec<[Vec<u8>; 2]>,
    pub god_item_names: Vec<Vec<u8>>,
    pub god_name_text: Vec<u8>,
    pub god_bought_texts: [Vec<u8>; 2],
    pub god_short_texts: [Vec<u8>; 2],
    pub god_chatter_texts: Vec<[Vec<u8>; 2]>,
    pub god_intro_texts: Vec<[Vec<u8>; 2]>,
    pub menu_texts: Vec<Option<Texture>>,
    pub map_ui_sheet: Option<Rc<Imgcut>>,
    pub map_reopen_times: BTreeMap<i32, f64>,
    pub item_drop_queue: Vec<Vec<i32>>,
    pub drop_icons: BTreeMap<i32, Option<Rc<Imgcut>>>,
    pub web_popup_entries: Vec<WebPopupEntry>,
    pub web_popup_shown: Vec<[i32; 2]>,
    pub item_snapshot: BTreeMap<i32, i32>,
    pub ad_button_id: i32,
    pub ad_button_cleared: u8,
    pub lineup_stages: BTreeMap<i16, Vec<i32>>,
    pub lineup_records: BTreeMap<i16, LineupRecord>,
    pub labyrinth_units: Vec<i32>,
    pub labyrinth_floors: BTreeMap<i32, LabyrinthFloor>,
    pub drop_items: BTreeMap<i32, DropRecord>,
    pub stage_rewards_taken: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub event_reward_cache: BTreeMap<i32, BTreeMap<i32, [u8; 4]>>,
    pub clear_count_rewards: BTreeMap<i32, BTreeMap<i32, Vec<[i32; 2]>>>,
    pub clear_lineups: BTreeMap<i32, Vec<Vec<[i32; 2]>>>,
    pub map_clear_counts: BTreeMap<i32, i32>,
    pub medals_awarded: Vec<i32>,
    pub enigma_durations: Vec<i32>,
    pub enigma: Enigma,
    pub map_open_neg4: Vec<i32>,
    pub map_open_neg9: Vec<i32>,
    pub map_open_neg10: Vec<i32>,
    pub map_open_neg11: Vec<i8>,
    pub map_open_neg16: Vec<i8>,
    pub map_open_neg17: Vec<i8>,
    pub map_open_neg18: Vec<i8>,
    pub map_open_neg20: Vec<i8>,
    pub map_open_neg22: Vec<Vec<i8>>,
    pub map_open_neg23: Vec<Vec<i8>>,
    pub map_open_neg24: Vec<Vec<i8>>,
    pub map_open_neg26: Vec<Vec<i8>>,
    pub xp_ad_maps: Vec<i32>,
    pub map_intervals: BTreeMap<i32, i32>,
    pub map_one_time: BTreeMap<i32, i32>,
    pub map_guerrilla_sets: BTreeMap<i32, i32>,
    pub aku_timers: BTreeMap<i32, f64>,
    pub altar_stage_values: BTreeMap<i32, i32>,
    pub stage_pair_progress: BTreeMap<i32, [i32; 2]>,
    pub units_dirty: u8,
    pub orb_inventory: BTreeMap<i32, i32>,
    pub stage_unlock_neg4: Vec<i32>,
    pub stage_unlock_neg9: Vec<i32>,
    pub stage_unlock_neg10: Vec<i32>,
    pub stage_unlock_neg11: Vec<i8>,
    pub stage_unlock_neg16: Vec<i8>,
    pub stage_unlock_neg17: Vec<i8>,
    pub stage_unlock_neg18: Vec<i8>,
    pub stage_unlock_neg19: Vec<i8>,
    pub stage_unlock_neg20: Vec<i8>,
    pub stage_unlock_neg22: Vec<Vec<i8>>,
    pub stage_unlock_neg23: Vec<Vec<i8>>,
    pub stage_unlock_neg24: Vec<Vec<i8>>,
    pub stage_unlock_neg26: Vec<Vec<i8>>,
    pub stage_unlock_cache: BTreeMap<i32, [i16; 4]>,
    pub stages_cleared_neg26: Vec<Vec<i8>>,
    pub stage_record_neg26: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg24: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg23: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg22: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg20: Vec<i16>,
    pub stage_record_neg19: Vec<i16>,
    pub stage_record_neg18: Vec<i16>,
    pub stage_record_neg17: Vec<i16>,
    pub stage_record_neg16: Vec<i16>,
    pub stage_record_neg11: Vec<i16>,
    pub stage_record_neg10: Vec<i32>,
    pub stage_record_neg9: Vec<i32>,
    pub stage_record_neg4: Vec<i32>,
    pub bgm_player_bound: bool,
    pub demon_banner_anim: Maanim,
    pub default_font: Vec<u8>,
    pub combo_banner_texts: [Option<Texture>; 3],
    pub crit_vfx_anim: Maanim,
    pub zkill_vfx_anim: Maanim,
    pub barrier_anims: [Maanim; 3],
    pub shield_anims: [Maanim; 5],
    pub savage_vfx: Vec<EffectSprite>,
    pub toxic_vfx: Vec<EffectSprite>,
    pub metal_killer_vfx: Vec<EffectSprite>,
    pub drain_vfx: Vec<EffectSprite>,
    pub savage_vfx_anim: Maanim,
    pub toxic_vfx_anim: Maanim,
    pub metal_killer_vfx_anim: Maanim,
    pub drain_vfx_anim: Maanim,
    sound: Option<Box<dyn SoundManager>>,
    text: Option<Box<dyn TextRenderer>>,
    platform: Option<Box<dyn Platform>>,
    ui: Option<Box<dyn UiHost>>,
    meta: Option<Box<dyn MetaHost>>,
    scene: Option<Box<dyn SceneHost>>,
    assets: Option<Box<dyn AssetSource>>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    pub const DECK_KEY: usize = 0x28;
    pub const DECK_STRIDE: usize = 0x2c;
    pub const ITEM_COUNTS_KIND_D: usize = 0x188;
    pub const ITEM_CF_COUNT: usize = 0x450;
    pub const ITEM_COUNTS_KIND_B: usize = 0x1084;
    pub const DECK_BUTTON_HELD: usize = 0x13dc;
    pub const DECK_TWO_LINES: usize = 0x13dd;
    pub const ITEM_COUNTS_KIND_9: usize = 0x13e2;
    pub const ITEM_COUNTS_KIND_9_STRIDE: usize = 0x18;
    pub const ITEM_7B_COUNT: usize = 0x1cc5;
    pub const ITEM_69_COUNT: usize = 0x1ec2;
    pub const ITEM_1D_COUNT: usize = 0x2628;
    pub const ITEM_91_COUNT: usize = 0x2630;
    pub const ITEM_9D_COUNT: usize = 0x2638;
    pub const ITEM_D4_COUNT: usize = 0x2640;
    pub const BATTLE_ZOOM_Y: usize = 0x3474;
    pub const ITEM_COUNTS_KIND_8: usize = 0x3478;
    pub const ITEM_COUNTS_KIND_A: usize = 0x3630;
    pub const ITEM_COUNTS_KIND_C: usize = 0x3640;
    pub const ITEM_16_COUNT: usize = 0xc158;
    pub const ITEM_6_COUNT: usize = 0xc2c8;
    pub const ITEM_7_COUNT: usize = 0xc2d0;
    pub const ITEM_COUNTS_KIND_3: usize = 0x4a460;
    pub const ITEM_14_COUNT: usize = 0x32cc64;
    pub const ITEM_15_COUNT: usize = 0x32cc6c;
    pub const ITEM_COUNTS_KIND_1: usize = 0x32cc74;
    pub const DEPLOY_NOTICE_TIMER: usize = 0x290550;
    pub const DEPLOY_NOTICE_KIND: usize = 0x290554;
    pub const BABY_BOOM_ACTIVE: usize = 0x32b6b4;
    pub const EVENT_POINT_BOOST: usize = 0x388028;
    pub const MEDAL_MONEY_0: usize = 0x19d0;
    pub const MEDAL_MONEY_1: usize = 0x19d4;
    pub const MEDAL_MONEY_4: usize = 0x19d8;
    pub const DEPLOY_LIMIT_RARITY_COUNTS: usize = 0x33b7f0;
    pub const DEPLOY_LIMIT_TOTAL: usize = 0x33b808;
    pub const ITEM_DEFINITIONS: usize = 0x38a9d4;
    pub const ITEM_REDIRECT_SCALES: usize = 0x38a9c8;
    pub const MISSION_CANNON_FIRED: usize = 0x3bb700;
    pub const ITEM_DEFINITION_STRIDE: usize = 0x40;
    pub const ITEM_5C_COUNT: usize = 0x427a58;
    pub const ITEM_COUNTS_KIND_5: usize = 0x440314;
    pub const ITEM_COUNTS_KIND_6: usize = 0x440344;
    pub const ITEM_COUNTS_KIND_7: usize = 0x44035c;
    pub const UNITS_OWNED: usize = 0x46ce8;
    pub const UNITS_OWNED_KEY: usize = 0x47a98;
    pub const UNIT_LEVELS: usize = 0x47a9c;
    pub const TECH_LEVELS: usize = 0x4a3ac;
    pub const WALLET_MONEY: usize = 0x4;
    pub const WALLET_WORKER_LEVEL: usize = 0xc;
    pub const WALLET_COOLDOWNS: usize = 0x14;
    pub const WALLET_COOLDOWN_KEY: usize = 0x3c;
    pub const WALLET_COOLDOWN_MAXES: usize = 0x40;
    pub const WALLET_COOLDOWN_MAX_KEY: usize = 0x68;
    pub const WALLET_CONJURE_READY: usize = 0x6c;
    pub const WALLET_CONJURE_LOCKOUT: usize = 0x94;
    pub const WALLET_CONJURE_TIMER: usize = 0xbc;
    pub const WALLET_SPIRIT_USED: usize = 0xe4;
    pub const WALLET_DEPLOY_COUNTS: usize = 0x108;
    pub const WALLET_ESCALATING_COSTS: usize = 0x130;
    pub const WALLET_SLOT_FLASH: usize = 0x180;
    pub const WALLET_CANNON_FIRED: usize = 0x1d0;
    pub const WALLET_SPAWN_SERIAL: usize = 0x1d4;
    pub const INPUT_BLOCKED: usize = 0x3265fc;
    pub const OPTION_MENU_IS_OPEN: usize = 0x326624;
    pub const SWIPE_DY: usize = 0x326640;
    pub const SWIPE_ANGLE: usize = 0x32664c;
    pub const SWIPE_VELOCITY: usize = 0x326650;
    pub const DECK_ROW_SWAP_DIRECTION: usize = 0x32665c;
    pub const CAMERA_KICK: usize = 0x326660;
    pub const SNIPER_TARGET: usize = 0x326680;
    pub const SNIPER_BOB_ANGLE: usize = 0x326684;
    pub const SNIPER_AIM_ANGLE: usize = 0x326688;
    pub const SNIPER_AIM_GOAL: usize = 0x32668c;
    pub const PENDING_STRIKE_TRIGGER_X: usize = 0x326690;
    pub const PENDING_STRIKE_Y: usize = 0x326758;
    pub const PENDING_STRIKE_TARGET: usize = 0x326820;
    pub const PENDING_STRIKE_ACTIVE: usize = 0x3268e8;
    pub const PENDING_STRIKE_SPEED: usize = 0x32691c;
    pub const PENDING_STRIKE_ANGLE: usize = 0x3269e4;
    pub const SNIPER_CHARGE: usize = 0x326aac;
    pub const SNIPER_FIRING: usize = 0x326ab0;
    pub const SNIPER_FIRE_FRAME: usize = 0x326ab4;
    pub const SNIPER_RECOIL: usize = 0x326ab8;
    pub const SNIPER_CASINGS: usize = 0x326abc;
    pub const SNIPER_CASINGS_LIVE: usize = 0x326fbc;
    pub const DECK_ROW_SHOWN: usize = 0x326fc0;
    pub const DECK_ROW_SWAP_OFFSETS: usize = 0x326fc4;
    pub const DECK_ROW_SWAP_FRAME: usize = 0x326fd4;
    pub const DECK_ROW_SWAP_TARGET: usize = 0x326fd8;
    pub const DECK_ROW_SWAPPING: usize = 0x326fdc;
    pub const DECK_SWIPE_LATCHED: usize = 0x326fde;
    pub const PINCH_ZOOMED: usize = 0x326fdf;
    pub const CAMERA_DRAGGING: usize = 0x326fe0;
    pub const PENDING_STRIKE_SPARKS: usize = 0x326b0c;
    pub const PENDING_STRIKE_SPARKS_STRIDE: usize = 0x18;
    pub const DRAW_TEMP_0: usize = 0x327da4;
    pub const DRAW_TEMP_1: usize = 0x327da8;
    pub const DRAW_TEMP_2: usize = 0x327dac;
    pub const DRAW_TEMP_3: usize = 0x327db0;
    pub const DRAW_TEMP_4: usize = 0x327db4;
    pub const DRAW_TEMP_5: usize = 0x327db8;
    pub const DRAW_TEMP_6: usize = 0x327dbc;
    pub const DRAW_TEMP_7: usize = 0x327dc0;
    pub const ANCHOR_OUT: usize = 0x33d0;
    pub const DRAW_LIST: usize = 0x4a598;
    pub const DRAW_SWAP: usize = 0x4aef8;
    pub const SNIPER_ALIGNED: usize = 0x327db0;
    pub const DECK_BAR_SLIDE: usize = 0x327dcc;
    pub const LETTERBOX_SHIFT: usize = 0x327f70;
    pub const OUTRO_EXIT_DIRECT: usize = 0x1ec0;
    pub const OUTRO_EXIT_EVENT: usize = 0x1ec1;
    pub const OUTRO_MAP_LOCKED: usize = 0x836c8;
    pub const OUTRO_VIDEO_BUTTON: usize = 0x858;
    pub const OUTRO_VIDEO_WATCHED: usize = 0x859;
    pub const LAST_VIDEO_TIME: usize = 0x860;
    pub const LEADERSHIP_REFUND: usize = 0x3f8;
    pub const LOSE_BANNER_Y: usize = 0x326648;
    pub const LOSE_CHOICE: usize = 0x327f78;
    pub const LOSE_NO_PRESS: usize = 0x328338;
    pub const LOSE_SHOP_PRESS: usize = 0x32836c;
    pub const LOSE_SHOP_X: usize = 0x326634;
    pub const LOSE_SHOP_RECT: usize = 0x3282e4;
    pub const LOSE_SHOP_HELD: usize = 0x32a412;
    pub const LOSE_RECORDED: usize = 0xc2dc;
    pub const LOSE_TIP: usize = 0x327f08;
    pub const EX_OFFERED: usize = 0x402160;
    pub const CAT_FOOD_SHOP_ENABLED: usize = 0xc2fc;
    pub const CAT_FOOD_SHOP_OPEN: usize = 0x32a442;
    pub const CAT_FOOD_SHOP_MODE: usize = 0x32a45c;
    pub const SHOP_TUTORIAL_SEEN: usize = 0x4a4b0;
    pub const PENDING_SCENE: usize = 0x3358;
    pub const SCENE_CHANGE_REQUESTED: usize = 0x3360;
    pub const REVIVE_REQUESTED: usize = 0x327f04;
    pub const FIRST_STAGE_WON: usize = 0xc2e4;
    pub const NEXT_STAGE_UNLOCKED: usize = 0x836c0;
    pub const WIN_TREASURE: usize = 0x836bc;
    pub const RANK_POPUP_SHOWN: usize = 0x38f58c;
    pub const RANK_REWARD_BASE: usize = 0x3bb708;
    pub const EX_ROLLED: usize = 0x402161;
    pub const EX_ACCEPTED: usize = 0x402162;
    pub const EX_MAP: usize = 0x402164;
    pub const EX_STAGE: usize = 0x40216c;
    pub const OUTRO_FRAME: usize = 0x836b4;
    pub const OUTRO_TICKS: usize = 0x3bb73c;
    pub const BATTLE_RESUMED: usize = 0x3880d8;
    pub const BATTLE_CONTINUED: usize = 0x32c970;
    pub const POINT_LIMIT_PENDING: usize = 0x220;
    pub const OUTRO_OK_PRESS: usize = 0x328334;
    pub const OUTRO_OK_RECT: usize = 0x328244;
    pub const OUTRO_OK_SLIDE: usize = 0x327dd0;
    pub const REWARD_POP_HOLD: usize = 0x328538;
    pub const LABYRINTH_RESULT_READY: usize = 0x1068;
    pub const LABYRINTH_RANK: usize = 0x1044;
    pub const LABYRINTH: usize = 0xad0;
    pub const LABYRINTH_MAP_ID: usize = 0x1020;
    pub const LABYRINTH_FLOOR_REACHED: usize = 0x103c;
    pub const LABYRINTH_FLOOR_BEST: usize = 0x1040;
    pub const DECK_BACK_ROW_ENABLED: usize = 0x32b392;
    pub const DRAG_LATCHED: usize = 0x32c8e4;
    pub const CPU_ENABLED: usize = 0x328560;
    pub const CPU_PENDING_ACTION: usize = 0x328564;
    pub const CPU_PICK: usize = 0x32856c;
    pub const CPU_USABLE: usize = 0x328574;
    pub const CPU_FACTION_STRIDE: usize = 0x28;
    pub const CPU_SAVING_FOR: usize = 0x3285c4;
    pub const CPU_CANNON_STATE: usize = 0x3285cc;
    pub const CPU_CANNON_WAIT: usize = 0x3285d4;
    pub const CPU_CANDIDATES: usize = 0x3285e4;
    pub const SCENE_0X64_PAGE: usize = 0x3284d4;
    pub const UI_TAP_LOCKOUT: usize = 0x32a474;
    pub const CAT_GOD_MENU_IS_OPEN: usize = 0x32b494;
    pub const CANNON_BLAST_ACTIVE: usize = 0x32b6ac;
    pub const BASE_GUARD_NOTICE: usize = 0x870;
    pub const KILLS_SINCE_SPAWN_TICK: usize = 0x10f8;
    pub const LABYRINTH_ACTIVE: usize = 0xff4;
    pub const SCORE_MODE_FLAG: usize = 0x32b9;
    pub const SCORE_TOTAL: usize = 0x32bc;
    pub const SCORE_ELAPSED: usize = 0x32c0;
    pub const SCORE_CHANGED: usize = 0x32e0;
    pub const SCORE_ANIM_TICK: usize = 0x32e4;
    pub const SCORE_SHOWN: usize = 0x32e8;
    pub const SCORE_FROM: usize = 0x32ec;
    pub const SCORE_ZOOM: usize = 0x32f0;
    pub const LINEUP_CANNON_TYPE: usize = 0x4b0;
    pub const LINEUP_CANNON_LEVEL: usize = 0x4b4;
    pub const EX_REDIRECT_A_BLOCKED: usize = 0x1490;
    pub const BUILT_DECK_EX_STAGE_KEY: usize = 0x3280;
    pub const USE_BUILT_DECK: usize = 0x3284;
    pub const STORY_MAP_COUNTS: usize = 0x3364;
    pub const UNIT_INFO_OVERLAY_OPEN: usize = 0x910;
    pub const CHAPTER_PROGRESS: usize = 0xc94c;
    pub const CHAPTER_PROGRESS_KEY: usize = 0xc974;
    pub const ENEMY_GUIDE_SEEN: usize = 0xd968;
    pub const PINCH: usize = 0x33d8;
    pub const CAMERA_ZOOM: usize = 0x340c;
    pub const TOUCH_X: usize = 0x3428;
    pub const TOUCH_START_X: usize = 0x3430;
    pub const TOUCH_PREV_X: usize = 0x3434;
    pub const TOUCH_Y: usize = 0x3438;
    pub const TOUCH_START_Y: usize = 0x3440;
    pub const TOUCH_PREV_Y: usize = 0x3444;
    pub const TOUCH_BEGAN: usize = 0x3448;
    pub const TOUCH_IS_DOWN: usize = 0x344a;
    pub const TOUCH_RELEASED: usize = 0x344b;
    pub const BACK_PRESSED: usize = 0x344e;
    pub const SCENE_ID: usize = 0x3450;
    pub const DECK_PRESETS: usize = 0xc310;
    pub const DECK_PRESET_STRIDE: usize = 0x2c;
    pub const DECK_PRESET_KEY: usize = 0xc338;
    pub const FACTION_1_DECK: usize = 0xc33c;
    pub const BATTLE_DECK: usize = 0xc6ac;
    pub const STAGES_CLEARED_CHAPTERS: usize = 0xc94c;
    pub const STAGE_RECORD_CHAPTERS: usize = 0xc978;
    pub const SEEN_ENEMIES: usize = 0xd968;
    pub const FACTION_1_UNIT_FORMS: usize = 0x495f4;
    pub const UNIT_FORMS: usize = 0x495fc;
    pub const BATTLE_FRAME_COUNTER: usize = 0x83678;
    pub const CAMERA_X: usize = 0x83688;
    pub const AUTO_CAMERA_MODE: usize = 0x836a4;
    pub const BATTLE_STATUS: usize = 0x836ac;
    pub const WORKER_UPGRADE_VFX: usize = 0x836d8;
    pub const CAMERA_MIN_ZOOM: usize = 0x836e4;
    pub const CASTLE_ID: usize = 0x836c4;
    pub const STAGE_CASTLE_ID: usize = 0x836fc;
    pub const TREASURE_PROGRESS: usize = 0x83708;
    pub const SPAWN_COUNTDOWN: usize = 0x838c0;
    pub const CAT_DEBRIS: usize = 0x9c768;
    pub const DEBRIS_STRIDE: usize = 0x10;
    pub const ENEMY_DEBRIS: usize = 0x9cae8;
    pub const CANNON_SHOTS: usize = 0x9ce68;
    pub const CANNON_SHOTS_FACTION_STRIDE: usize = 0xb4;
    pub const CANNON_SHOT_STRIDE: usize = 0xc;
    pub const STAGE_LENGTH: usize = 0x9e528;
    pub const STAGE_SPAWN_MIN: usize = 0x9e530;
    pub const STAGE_SPAWN_MAX: usize = 0x9e534;
    pub const STAGE_BACKGROUND_ID: usize = 0x9e538;
    pub const STAGE_MAX_ENEMIES: usize = 0x9e53c;
    pub const CASTLE_ENEMY_ROW: usize = 0x9e540;
    pub const STAGE_SCORE_TIME_LIMIT: usize = 0x9e544;
    pub const STAGE_BOSS_GUARD: usize = 0x9e548;
    pub const SCENE_0X63_STATE: usize = 0x325c2c;
    pub const INSETS_IGNORED: usize = 0x20d8;
    pub const STAGE_INDEX: usize = 0x325c48;
    pub const CHAPTER_MODE: usize = 0x327efc;
    pub const FACTION_1_BUTTON_ROWS: usize = 0x327f18;
    pub const POWERUPS: usize = 0x327eb0;
    pub const POWERUP_AVAILABLE: usize = 0x327eec;
    pub const SPEED_UP_LATCH: usize = 0x327eef;
    pub const BUTTON_UNIT_FORMS: usize = 0x327f40;
    pub const BATTLE_IS_OUTBREAK: usize = 0x32c5c8;
    pub const OUTBREAKS_ENABLED: usize = 0x32c5c9;
    pub const BATTLE_IS_INVASION: usize = 0x32c5d6;
    pub const BATTLE_IS_Z_INVASION: usize = 0x32c5d7;
    pub const INVASION_STAGE: usize = 0x32c5d8;
    pub const MAP_NEG15_CLEARED: usize = 0x32c5d9;
    pub const MAP_NEG25_CLEARED: usize = 0x32c5da;
    pub const WAVE_RECORDS: usize = 0x333d6c;
    pub const WAVE_RECORD_STRIDE: usize = 0x30;
    pub const WAVE_SPRITES: usize = 0x3362ec;
    pub const MAP_INDEX: usize = 0x3388b8;
    pub const STAGES_CLEARED_STORY: usize = 0x33e020;
    pub const STAGES_CLEARED_NEG6: usize = 0x340748;
    pub const STAGE_RECORD_STORY: usize = 0x340818;
    pub const STAGE_RECORD_NEG6: usize = 0x37b1b0;
    pub const SAVED_MAP_TYPE: usize = 0x3836b4;
    pub const CHAPTER_COST_TIER: usize = 0x388028;
    pub const PROC_ROLLS: usize = 0x3880dc;
    pub const WAVE_HITS: usize = 0x388110;
    pub const SELECTED_DECK_PRESET: usize = 0x38fd6c;
    pub const PRESET_STYLE_PARTS: usize = 0x427259;
    pub const PRESET_FOUNDATION_PARTS: usize = 0x42725a;
    pub const STAR_LEVEL: usize = 0x38fecc;
    pub const EX_MAP_INDEX: usize = 0x402168;
    pub const EX_STAGE_INDEX: usize = 0x402170;
    pub const UNIT_LEVEL_CURVE: usize = 0x4475a4;
    pub const UNIT_EXP_CURVE: usize = 0x458764;
    pub const STAGE_NO_CONTINUES: usize = 0x46a9b4;
    pub const STAGE_EX_CHANCE: usize = 0x46a9b8;
    pub const STAGE_EX_MAP: usize = 0x46a9bc;
    pub const STAGE_EX_STAGE_MIN: usize = 0x46a9c0;
    pub const STAGE_EX_STAGE_MAX: usize = 0x46a9c4;
    pub const STAGE_RECORD_NEG8: usize = 0x46a9c8;
    pub const ITEM_DEFINITION_IDS: usize = 0x38a9c0;
    pub const DECK_HOLD_FRAMES: usize = 0x13d0;
    pub const DECK_HOLD_SLOT: usize = 0x13d4;
    pub const DECK_HOLD_RELEASE: usize = 0x13d8;
    pub const VIBRATION_ENABLED: usize = 0x221;
    pub const MAP_STAGE_ROWS: usize = 0x3836b8;
    pub const MAP_STAGE_ROW_STRIDE: usize = 0xbc;
    pub const STAGE_ROW: usize = 0x83674;
    pub const STAGE_MUSIC_ROW: usize = 0x3bbd58;
    pub const BGM_SWITCH_STATE: usize = 0x29023c;
    pub const BGM_SWITCH_FRAME: usize = 0x290240;
    pub const BGM_SWITCH_FRAMES: usize = 0x290244;
    pub const BGM_SWITCH_NEXT: usize = 0x290248;
    pub const BGM_BOSS_PHASE: usize = 0x29024c;
    pub const BGM_PENDING: usize = 0x46b958;
    pub const BGM_DELAY_FRAME: usize = 0x46b95c;
    pub const BGM_DELAY: usize = 0x46b960;
    pub const ITEMS_SELECTED: usize = 0x327ed8;
    pub const ITEMS_SELECTED_LABYRINTH: usize = 0x2374;
    pub const ITEMS_SELECTED_SCORE_MODE: usize = 0x2367;
    pub const DEMON_BANNER_FRAME: usize = 0x12b8;
    pub const COMBO_BANNER_STEP_FRAMES: usize = 0x440be8;
    pub const COMBO_BANNER_LIFETIME: usize = 0x440bec;
    pub const COMBO_BANNER_SPEED: usize = 0x440bf0;
    pub const COMBO_BANNER_TEXT_STEP: usize = 0x440bf4;
    pub const COMBO_BANNER_STATE: usize = 0x440bf8;
    pub const COMBO_BANNER_PHASE: usize = 0x440bfc;
    pub const COMBO_BANNER_X: usize = 0x440c00;
    pub const COMBO_BANNER_SUB: usize = 0x440c04;
    pub const COMBO_BANNER_TICKS: usize = 0x440c08;
    pub const COMBO_SKIP_RECT: usize = 0x328224;
    pub const COMBO_BANNER_UNITS: usize = 0x440c0c;
    pub const CRIT_VFX: usize = 0x9cfd0;
    pub const CRIT_VFX_STRIDE: usize = 0x10;
    pub const ZKILL_VFX: usize = 0x9dc50;
    pub const ZKILL_VFX_STRIDE: usize = 0x10;
    pub const BARRIER_VFX: usize = 0x9de30;
    pub const BARRIER_VFX_STRIDE: usize = 0x1c;
    pub const SHIELD_VFX: usize = 0x9e178;
    pub const SHIELD_VFX_STRIDE: usize = 0x1c;
    pub const BASE_GUARD_NOTICE_FRAME: usize = 0x874;
    pub const TUTORIAL_POPUP_OPEN: usize = 0x32b44c;
    pub const OPTION_WINDOW: usize = 0x469928;
    pub const UNIT_INFO_SLOT: usize = 0x914;
    pub const CURTAIN_ACTIVE: usize = 0x326618;
    pub const CURTAIN_STYLE: usize = 0x326620;
    pub const BATTLE_TICKS: usize = 0x328524;
    pub const BATTLE_INTRO_FRAME: usize = 0x32c798;
    pub const CAMERA_MATRIX: usize = 0x3410;
    pub const CAMERA_OFFSET: usize = 0x3420;
    pub const BASE_KILL_BLOCKED: usize = 0x392778;
    pub const AUTO_CAMERA_ARRIVED: usize = 0x327f68;
    pub const SETUP_FRAMES: usize = 0x328510;
    pub const REWARD_POP_COUNTER: usize = 0x327ef8;
    pub const SPEED: usize = 0x327da0;
    pub const CAT_GOD_HEAL_PENDING: usize = 0x32b6a8;
    pub const BABY_BOOM_FRAMES: usize = 0x327ef4;
    pub const DEPLOY_FULL_FLASH: usize = 0x1ce0;
    pub const HIT_COUNT: usize = 0x28be38;
    pub const HIT_LIST: usize = 0x28bc98;
    pub const HIT_SWAP: usize = 0x28be30;
    pub const OUTRO_PHASE: usize = 0x836b0;
    pub const LOSE_TIP_SHOWN: usize = 0x836d4;
    pub const LOST_MAP_TYPE: usize = 0x327f0c;
    pub const LOST_MAP_INDEX: usize = 0x327f10;
    pub const LOST_STAGE: usize = 0x327f14;
    pub const DEFEAT_COUNTER: usize = 0x2f14;
    pub const EVENT_UNIT_OWNED: usize = 0x38ef44;
    pub const NEW_BEST_SCORE: usize = 0x836d0;
    pub const PLAY_FRAMES: usize = 0x836ec;
    pub const OPTION_WINDOW_KIND: usize = 0x46992c;
    pub const BG_IMAGE_ID: usize = 0x32b7e4;
    pub const BG_MODEL_ID: usize = 0x32b7dc;
    pub const TECH_MAX_LEVELS: usize = 0x38a8f0;
    pub const UNIT_FORM_COUNTS: usize = 0x3fb178;
    pub const COMBO_PAGE_LIST: usize = 0x32c270;
    pub const COMBO_UNLOCK_NOTICE: usize = 0x440c94;
    pub const COMBO_TAB: usize = 0x440b8c;
    pub const LEGEND_STAGE_ENERGY: usize = 0x46a824;
    pub const LEGEND_STAGE_INFO: usize = 0x46a054;
    pub const BG_SETUP: usize = 0x32b7cc;
    pub const BG_TINT_XS: usize = 0x328670;
    pub const BG_TINT_YS: usize = 0x328680;
    pub const BG_TINT_COLORS: usize = 0x328690;
    pub const OUTRO_RECTS: usize = 0x3282d4;
    pub const OPTION_RECTS: usize = 0x3282a4;
    pub const DECK_SWAP_BLOCK: usize = 0x328650;
    pub const UI_STATE_TAIL: usize = 0x32b7c8;
    pub const HUD_RECTS: usize = 0x3283d0;
    pub const TUTORIAL_TIMER_TAIL: usize = 0x32b460;
    pub const RESTRICTION_WARNING_TEXTS: usize = 0xb800;
    pub const STAGE_RECORD_CHAPTERS_KEY: usize = 0xca44;
    pub const STAGE_CLEAR_FLAG: usize = 0xc2e0;
    pub const POWERUP_USED: usize = 0x3bb701;
    pub const STAGE_BASE_HP: usize = 0x9e52c;
    pub const BATTLE_CLOCK: usize = 0x9e520;
    pub const CAT_GOD_GLOW_TIMER: usize = 0x32b48c;
    pub const LINEUP_BASE_LEVEL: usize = 0x4b8;
    pub const POWERUP_FREE: usize = 0x327ee2;
    pub const POWERUP_GRANTS: usize = 0x13e0;
    pub const RESUMED_MONEY: usize = 0x8369c;
    pub const BATTLE_DECK_KEY: usize = 0xc6d4;
    pub const EX_REDIRECT_ENABLED: usize = 0x3bc491;
    pub const SCENE_4_PAGE: usize = 0x32f4;
    pub const ALL_MAPS_OPEN: usize = 0x38fc10;
    pub const MAP_COORDS: usize = 0x290250;
    pub const POLYGON_YS: usize = 0x3286ac;
    pub const POLYGON_XS: usize = 0x3286a0;
    pub const LETTERBOX_PAD: usize = 0x327f74;
    pub const MIRACLE_PRICES: usize = 0x46b970;
    pub const CAT_GOD_TICKS: usize = 0x32b4ac;
    pub const CAT_GOD_DROP_SPEED: usize = 0x32b534;
    pub const CAT_GOD_FLASH_X: usize = 0x32b518;
    pub const CAT_GOD_RETURN_STEP: usize = 0x32b508;
    pub const CAT_GOD_RETURN_TICKS: usize = 0x32b504;
    pub const CAT_GOD_SAVED_CAMERA_X: usize = 0x32b500;
    pub const CAT_GOD_SAVED_ZOOM: usize = 0x32b4fc;
    pub const CAT_GOD_ZOOM_DELTA: usize = 0x32b4f4;
    pub const CAT_GOD_ZOOM_TICKS: usize = 0x32b4f8;
    pub const CAT_GOD_ZOOM_DONE: usize = 0x32b50c;
    pub const CAT_GOD_FADE: usize = 0x32b4a4;
    pub const CAT_GOD_PUSHING: usize = 0x32b6b0;
    pub const CAT_GOD_SHAKE_Y: usize = 0x32b58c;
    pub const CAT_GOD_SHAKE_X: usize = 0x32b53c;
    pub const CAT_GOD_FRAMES: usize = 0x32b4c0;
    pub const CAT_GOD_STATUE_COLUMN: usize = 0x32b53c;
    pub const CAT_GOD_STATUE_ROW: usize = 0x32b58c;
    pub const CAT_GOD_ANIM_FRAME: usize = 0x32b4bc;
    pub const CAT_GOD_HOVER: usize = 0x32a427;
    pub const CAT_GOD_BACK_RECT: usize = 0x32b64c;
    pub const CAT_GOD_CONFIRM_RECT: usize = 0x32b63c;
    pub const CAT_GOD_CLOSE_RECT: usize = 0x32b62c;
    pub const CAT_GOD_MIRACLE_RECTS: usize = 0x32b5ec;
    pub const CAT_GOD_CONFIRM_OPEN: usize = 0x32b495;
    pub const CAT_GOD_SELECTED: usize = 0x32b6a4;
    pub const CAT_GOD_PRESSES: usize = 0x32b680;
    pub const CAT_GOD_OPEN_TICKS: usize = 0x32b514;
    pub const CAT_GOD_IDLE_FRAMES: usize = 0x32b510;
    pub const CAT_GOD_BOB: usize = 0x32b528;
    pub const CAT_GOD_VELOCITY: usize = 0x32b538;
    pub const CAT_GOD_OFFSET: usize = 0x32b524;
    pub const CAT_GOD_SETTLE_FRAME: usize = 0x32b4b8;
    pub const CAT_GOD_CHATTER_TIMER: usize = 0x32b4b0;
    pub const CAT_GOD_SPIN_SPEED: usize = 0x32b49c;
    pub const CAT_GOD_STATE: usize = 0x32b4a8;
    pub const DECK_COOLDOWN_VFX: usize = 0x2778;
    pub const EFFECT_SLOTS: usize = 0x33356c;
    pub const BGM_PLAYER: usize = 0x46b948;
    pub const LEADERSHIP_TOTAL: usize = 0x3f4;
    pub const OUTRO_EXIT_TARGET: usize = 0xc30c;
    pub const LOSE_ENTRY_CHAPTER: usize = 0xc300;
    pub const MENU_BUILD_MODE: usize = 0x32e794;
    pub const MENU_CURSOR: usize = 0x440da0;
    pub const MENU_TEXTURE_PAGE: usize = 0x440d88;
    pub const MAP_RETURN_FLAG: usize = 0x3388c0;
    pub const HUD_STATE: usize = 0x3284a0;
    pub const SWIPE_STATE: usize = 0x326658;
    pub const SCROLL_STATE: usize = 0x32662c;
    pub const FADE_MENU_STATE: usize = 0x326628;
    pub const SCENE_0X64_PAGE_NEXT: usize = 0x3284dc;
    pub const FADE_STARTED: usize = 0x2228;
    pub const FADE_FRAME: usize = 0x32661c;
    pub const MAP_ENTRY_FLAGS: usize = 0x329404;
    pub const SCENE_LATCHES: usize = 0x32c730;
    pub const ITEM_HOLD_SCORE_MODE: usize = 0x2360;
    pub const ITEM_HOLD_LABYRINTH: usize = 0x236d;
    pub const ITEM_HOLD_NORMAL: usize = 0x32c644;
    pub const POWERUP_CLEARED: usize = 0x327ef0;
    pub const LABYRINTH_RANKING: usize = 0x1070;
    pub const LEADERSHIP_NOTICE: usize = 0x3259;
    pub const STAMINA_HALVED: usize = 0x32c8eb;
    pub const DROP_MAP_STAGES: usize = 0x381e60;
    pub const SPECIAL_BEST_SCORES: usize = 0x3824a0;
    pub const FESTIVAL_STAGES: usize = 0x32c708;
    pub const TREASURE_FESTIVAL_ENABLED: usize = 0x32c8e5;
    pub const STAGE_SCORE: usize = 0x836cc;
    pub const CHAPTER_BEST_SCORES: usize = 0x3bbaf0;
    pub const RANKING_BEST_SCORES: usize = 0x3afa50;
    pub const RANKING_ID: usize = 0x3bb704;
    pub const SCORE_RANK_STATUSES: usize = 0x3bb710;
    pub const MAP_STAGE_SET: usize = 0x38eea4;
    pub const MAP_DATA_ID: usize = 0x38eea0;
    pub const TUTORIAL_STAGE_SIX: usize = 0x447508;
    pub const FIRST_WIN_PENDING: usize = 0x222;
    pub const FIRST_WIN_GATE_B: usize = 0xc8a4;
    pub const FIRST_WIN_GATE_A: usize = 0x4a560;
    pub const EVENT_REWARD_ID: usize = 0x3bb698;
    pub const REWARD_STATUS: usize = 0x3bb70c;
    pub const DROP_FLAG: usize = 0x32c96c;
    pub const DROP_ROLL: usize = 0x32c968;
    pub const DROP_RATE: usize = 0x32c964;
    pub const UNIT_UNLOCKED_BY_CLEAR: usize = 0x32c958;
    pub const UNIT_UNLOCK_NOTICE: usize = 0x4af04;
    pub const OUTRO_NEW_CLEAR: usize = 0x32c95c;
    pub const OUTRO_STAGE_CLEARED: usize = 0x836f0;
    pub const OUTRO_ENTRY_STAGE: usize = 0x325c3c;
    pub const OUTRO_CHAPTER_MODE: usize = 0x327f00;
    pub const AD_CONFIRM_DECLINED: usize = 0x8d8;
    pub const WIN_XP: usize = 0x836b8;
    pub const PRESET_CANNON_PARTS: usize = 0x427258;
    pub const BATTLE_LINEUP: usize = 0xc680;
    pub const LABYRINTH_UNIT_COUNT: usize = 0xe10;
    pub const LABYRINTH_FLOOR_RESULT: usize = 0xe0c;
    pub const EVENT_REWARDS_NEG2: usize = 0x3bb5d0;
    pub const EVENT_REWARDS: usize = 0x3aaf38;
    pub const CLEAR_LINEUP: usize = 0xc654;
    pub const TREASURE_LEVELS_STRIDE: usize = 0xc8;
    pub const TREASURE_LEVELS: usize = 0xd198;
    pub const REPLAY_MODE: usize = 0x38fed0;
    pub const FESTIVAL_COTC: usize = 0x32c814;
    pub const FESTIVAL_ITF: usize = 0x32c808;
    pub const FESTIVAL_EOC: usize = 0x32c7fc;
    pub const MAP_OPEN_NEG6: usize = 0x37dce8;
    pub const MAP_OPEN_STORY: usize = 0x37b5c0;
    pub const REWARD_FORMS_OWNED: usize = 0x3aa158;
    pub const REWARD_UNITS_OWNED: usize = 0x3a93a8;
    pub const UNITS_UNLOCKED_FLAG: usize = 0x4a408;
    pub const REWARD_1002_OWNED: usize = 0x32cbc4;
    pub const STAGE_UNLOCK_CHAPTERS: usize = 0xc924;
    pub const STAGE_UNLOCK_NEG3: usize = 0xc934;
    pub const STAGE_UNLOCK_NEG7: usize = 0xc940;
    pub const STAGE_UNLOCK_NEG6: usize = 0x33df38;
    pub const STAGE_UNLOCK_STORY: usize = 0x33b810;
    pub const WALLET_RED_GAUGE_FRAMES: usize = 0x1a8;
    pub const WALLET_ORB_DEPLOYS_SEEN: usize = 0x158;
    pub const DRAW_FRAMES: usize = 0x327f84;
    pub const BLINK_COUNTER: usize = 0x327f7c;
    pub const BLINK_ON: usize = 0x327f80;
    pub const CAT_GOD_SPIN: usize = 0x32b498;
    pub const CAT_GOD_GLOW: usize = 0x32b4a0;
    pub const TUTORIAL_CLEARED: usize = 0xc2d8;
    pub const TUTORIAL_STEP: usize = 0x224;
    pub const TUTORIAL_DECK_SEEN: usize = 0x4a49c;
    pub const TUTORIAL_TWO_ROWS_SEEN: usize = 0x4a4f0;
    pub const TUTORIAL_TIMER: usize = 0x32b450;
    pub const TUTORIAL_CAT_GOD_SEEN: usize = 0x4a4a0;
    pub const CAT_GOD_AVAILABLE: usize = 0xc2f4;
    pub const ENTRY_STAGE: usize = 0x325c38;
    pub const CAT_GOD_BUTTON_PRESS: usize = 0x32b67c;
    pub const CAT_GOD_INTRO_STEP: usize = 0x4a4b4;
    pub const PAUSE_PRESS: usize = 0x328340;
    pub const SPEED_UP_PRESS: usize = 0x327e84;
    pub const CPU_PRESS: usize = 0x327e90;
    pub const SNIPER_PRESS: usize = 0x327e98;
    pub const DECK_PRESS: usize = 0x328380;
    pub const CAT_GOD_BUTTON_SINK: usize = 0x32b520;
    pub const CAT_GOD_BUTTON_RECT: usize = 0x32b5dc;
    pub const TOUCH_CAPTURED: usize = 0x326fdd;
    pub const ITEM_RECTS: usize = 0x327dd4;
    pub const TOOLTIP_ITEM: usize = 0x326600;
    pub const TOOLTIP_PAGE: usize = 0x32a470;
    pub const CANNON_RECT: usize = 0x328204;
    pub const WORKER_RECT: usize = 0x328214;
    pub const PAUSE_RECT: usize = 0x328234;
    pub const CANNON_HELD: usize = 0x32a404;
    pub const WORKER_HELD: usize = 0x32a405;
    pub const PAUSE_HELD: usize = 0x32a407;
    pub const DECK_BUTTON_PRESSED: usize = 0x32a408;
    pub const BG_PARTICLES: usize = 0x28be44;
    pub const BG_DRIFTERS: usize = 0x28c6dc;
    pub const BG_STARS: usize = 0x28c8bc;
    pub const BG_SPRITES: usize = 0x28cd1c;

    pub fn new() -> Self {
        Self {
            raw: vec![0u8; SIZE].into_boxed_slice(),
            stage_enemies: Vec::new(),
            spawn_states: Vec::new(),
            unit_models: [Vec::new(), Vec::new()],
            unit_anims: [Vec::new(), Vec::new()],
            death_surge_anims: Default::default(),
            counter_surge_anims: Default::default(),
            wave_anim: Default::default(),
            mini_wave_anim: Default::default(),
            effect_anims: Default::default(),
            base_anims: Default::default(),
            battle_event_latch: Default::default(),
            event_items: None,
            listed_item_counts: Vec::new(),
            metal_killer_map: Default::default(),
            surge_events: Vec::new(),
            counter_surge_events: Vec::new(),
            explosion_events: Vec::new(),
            base_shake: Default::default(),
            attackers_by_serial: Default::default(),
            scored_maps: Default::default(),
            cleared_session_keys: Vec::new(),
            conditioned_maps: Vec::new(),
            stage_conditions: BTreeMap::new(),
            legend_stage_conditions: Vec::new(),
            aku_stage_lists: BTreeMap::new(),
            unlock_groups: BTreeMap::new(),
            unlock_flags: BTreeMap::new(),
            condition_list_200k: Vec::new(),
            condition_list_300k: Vec::new(),
            deploy_queue: Vec::new(),
            altar_level_caps: BTreeMap::new(),
            altar_unsealed: BTreeMap::new(),
            screen_metrics: ScreenMetrics::default(),
            deck_bar_base_y: 0x220,
            stage_restrictions: BTreeMap::new(),
            cannon_type_names: BTreeMap::new(),
            enemy_names: Vec::new(),
            cat_names: Vec::new(),
            item_names: Vec::new(),
            item_descriptions: Vec::new(),
            unit_info_texts: [None; 4],
            localizable: BTreeMap::new(),
            combo_names: Vec::new(),
            combo_effect_texts: Vec::new(),
            combo_power_texts: Vec::new(),
            map_records: BTreeMap::new(),
            bg_effects: BgEffects::default(),
            bg_anim_cache: BTreeMap::new(),
            dialogs: DialogManager::default(),
            buttons: ButtonBank::default(),
            unlock_popups: BTreeMap::new(),
            img001_sheet: None,
            outro_event_sheets: Default::default(),
            texture_cache: BTreeMap::new(),
            reward_queue: Vec::new(),
            item_possession: BTreeMap::new(),
            treasure_names: Vec::new(),
            stage_names: Vec::new(),
            next_stage_names: Vec::new(),
            next_stage_caption: Vec::new(),
            ex_stage_names: Vec::new(),
            ex_groups: BTreeMap::new(),
            ex_lottery: Vec::new(),
            hidden_drop_keys: BTreeMap::new(),
            stage_pair_records: BTreeMap::new(),
            altar_enemy_ids: Default::default(),
            altar_rewards: BTreeMap::new(),
            label_texts: vec![None; 0x434],
            warning2_rows: vec![Default::default(); 0x73],
            lose_rows: Vec::new(),
            lose_row_settings: Vec::new(),
            lose_text_settings: Vec::new(),
            web_popup_pending: BTreeMap::new(),
            img004_sheet: None,
            reward_defs: BTreeMap::new(),
            reward_claimed: BTreeMap::new(),
            release_points: BTreeMap::new(),
            server_flags: BTreeMap::new(),
            event_unit_rows: Vec::new(),
            drop_chara_max_1000: -1,
            drop_chara_max_1100: -1,
            img039_sheet: None,
            deck_button_x: [
                0x9f, 0x121, 0x1a3, 0x225, 0x2a7, 0xab, 0x12d, 0x1af, 0x231, 0x2b3,
            ],
            tooltip_texts: [None; 16],
            enemy_kill_counts: BTreeMap::new(),
            best_scores: BTreeMap::new(),
            ranking_entries: Vec::new(),
            battle_effects: BattleEffects::default(),
            boss_shockwave_anim: Maanim::default(),
            cannon_part_rows: Default::default(),
            maps_neg26: Default::default(),
            maps_neg24: Default::default(),
            maps_neg23: Default::default(),
            maps_neg22: Default::default(),
            maps_neg21: Default::default(),
            maps_neg20: Default::default(),
            maps_neg19: Default::default(),
            maps_neg18: Default::default(),
            maps_neg17: Default::default(),
            maps_neg16: Default::default(),
            maps_neg11: Default::default(),
            maps_neg10: Default::default(),
            maps_neg9: Default::default(),
            maps_neg4: Default::default(),
            ex_option_targets: Default::default(),
            ex_replacement_stages: Default::default(),
            built_deck_stages: Default::default(),
            built_deck_records: Default::default(),
            cannon_parts: Default::default(),
            enemy_castle: Vec::new(),
            fixed_lineup_store: Default::default(),
            combo_store: Default::default(),
            chara_groups: Default::default(),
            star_multipliers: Default::default(),
            map_cost_multipliers: Default::default(),
            settings: Default::default(),
            treasure_store: Default::default(),
            orb_store: Default::default(),
            special_rules: Default::default(),
            equipped_orbs: Default::default(),
            map_data: Default::default(),
            map_data_ids: Default::default(),
            map_stage_sets: Default::default(),
            talent_definitions: Default::default(),
            talent_levels: Default::default(),
            outbreak_active: Default::default(),
            outbreak_cleared: Default::default(),
            cleared_map_ids: Default::default(),
            stages_cleared_cache: Default::default(),
            stages_cleared_neg24: Default::default(),
            stages_cleared_neg23: Default::default(),
            stages_cleared_neg22: Default::default(),
            stages_cleared_neg20: Default::default(),
            stages_cleared_neg18: Default::default(),
            stages_cleared_neg17: Default::default(),
            stages_cleared_neg16: Default::default(),
            stages_cleared_neg11: Default::default(),
            stages_cleared_neg10: Default::default(),
            stages_cleared_neg9: Default::default(),
            stages_cleared_neg4: Default::default(),
            stage_record_cache: Default::default(),
            download_sheet: Default::default(),
            deploy_cost_alt_sheet: Default::default(),
            scene_img005_sheet: Default::default(),
            trait_icons: Default::default(),
            ability_icons: Default::default(),
            picture_book_abilities: Default::default(),
            picture_book_traits: Default::default(),
            picture_book_trait_order: Default::default(),
            unit_icon_textures: Default::default(),
            enemy_sheets: Default::default(),
            unit_sheets: Default::default(),
            enemy_castle_sheet: Default::default(),
            base_models: Default::default(),
            base_sheets: (0..4).map(|_| cell::Cell::new(None)).collect(),
            bg_effect_sheets: Default::default(),
            option_window_sheet: Default::default(),
            effect_models: Default::default(),
            effect_sheets: Default::default(),
            smallvolcano_anims: Default::default(),
            volcano_anims: Default::default(),
            warp_anims: Default::default(),
            explosion_anims: Default::default(),
            castle_anims: Default::default(),
            castle_models: Default::default(),
            skill_effect_invalid_anim: Default::default(),
            skill_wave_stop_e_anim: Default::default(),
            skill_wave_invalid_e_anim: Default::default(),
            skill_down_e_anim: Default::default(),
            skill_shield_e_anim: Default::default(),
            skill_stop_e_anim: Default::default(),
            skill_slow_e_anim: Default::default(),
            skill_up_e_anim: Default::default(),
            skill_wave_stop_anim: Default::default(),
            skill_wave_invalid_anim: Default::default(),
            skill_down_anim: Default::default(),
            skill_shield_anim: Default::default(),
            skill_stop_anim: Default::default(),
            skill_slow_anim: Default::default(),
            skill_up_anim: Default::default(),
            invoke_equipment_anim: Default::default(),
            fever_anim: Default::default(),
            attack_invalid_anim: Default::default(),
            guard_e_breaker_anim: Default::default(),
            guard_e_anim: Default::default(),
            smallwave_attack_e_anim: Default::default(),
            wave_attack_e_anim: Default::default(),
            skill_curse_e_anim: Default::default(),
            skill_curse_anim: Default::default(),
            zombie_back_anim: Default::default(),
            zombie_revive_anim: Default::default(),
            zombie_down_anim: Default::default(),
            skill_effect_invalid_model: Default::default(),
            skill_wave_stop_e_model: Default::default(),
            skill_wave_invalid_e_model: Default::default(),
            skill_down_e_model: Default::default(),
            skill_shield_e_model: Default::default(),
            skill_stop_e_model: Default::default(),
            skill_slow_e_model: Default::default(),
            skill_up_e_model: Default::default(),
            skill_wave_stop_model: Default::default(),
            skill_wave_invalid_model: Default::default(),
            skill_down_model: Default::default(),
            skill_shield_model: Default::default(),
            skill_stop_model: Default::default(),
            skill_slow_model: Default::default(),
            skill_up_model: Default::default(),
            demonbattle_model: Default::default(),
            invoke_equipment_model: Default::default(),
            fever_model: Default::default(),
            recast_decrease_e_model: Default::default(),
            metal_strong_model: Default::default(),
            smallvolcano_e_model: Default::default(),
            smallvolcano_model: Default::default(),
            volcano_e_model: Default::default(),
            volcano_model: Default::default(),
            percentage_attack_model: Default::default(),
            attack_invalid_model: Default::default(),
            strong_attack_model: Default::default(),
            warp_chara_model: Default::default(),
            warp_model: Default::default(),
            demonsummon_e_model: Default::default(),
            demonsummon_model: Default::default(),
            demonsoul_01_model: Default::default(),
            demonsoul_00_model: Default::default(),
            guard_e_model: Default::default(),
            demonshield_model: Default::default(),
            barrier_model: Default::default(),
            explosion_e_model: Default::default(),
            explosion_model: Default::default(),
            smallwave_attack_e_model: Default::default(),
            smallwave_attack_model: Default::default(),
            wave_attack_e_model: Default::default(),
            wave_attack_model: Default::default(),
            skill_curse_e_model: Default::default(),
            skill_curse_model: Default::default(),
            skill_zombie_strong_model: Default::default(),
            zombie_model: Default::default(),
            crit_vfx_model: Default::default(),
            boss_welcome_model: Default::default(),
            sealed_announce_sheets: Default::default(),
            demonsoul_sheets: Default::default(),
            skill_sheets: Default::default(),
            zombie_sheets: Default::default(),
            equipment_effect_s_sheet: Default::default(),
            equipment_attribute_s_sheet: Default::default(),
            equipment_shadow_sheet: Default::default(),
            equipment_grade_sheet: Default::default(),
            equipment_effect_sheet: Default::default(),
            equipment_attribute_sheet: Default::default(),
            img015_sheet: Default::default(),
            fever_sheet: Default::default(),
            effect_a_sheet: Default::default(),
            castle_sheet: Default::default(),
            bubble_sheet: Default::default(),
            img001_second_sheet: Default::default(),
            img101_sheet: Default::default(),
            img100_sheet: Default::default(),
            img042_sheet: Default::default(),
            img041_sheet: Default::default(),
            img040_sheet: Default::default(),
            img006_sheet: Default::default(),
            img043_sheet: Default::default(),
            stage_name_sheet: Default::default(),
            img060_sheet: Default::default(),
            img024_sheet: Default::default(),
            bg_sheet: Default::default(),
            combo_definitions: Default::default(),
            play_dungeon_rows: Default::default(),
            restriction_warning_texts: Default::default(),
            bg_anim_names: Default::default(),
            bg_model_names: Default::default(),
            bg_models: Default::default(),
            castle_hp_growth: Default::default(),
            dungeon_clear_counts: Default::default(),
            random_dungeon_rows: Default::default(),
            entry_record_maps: Default::default(),
            data_pack_expected: Default::default(),
            data_pack_key: Default::default(),
            map_stage_options: Default::default(),
            ranking_maps: Default::default(),
            event_reward_maps: Default::default(),
            text_blocks: Default::default(),
            mapicon_sheet: Default::default(),
            img003_sheet: Default::default(),
            img002_sheet: Default::default(),
            draw: Default::default(),
            miracle_anims: Default::default(),
            miracle_levels: Default::default(),
            battle_option_texts: vec![Vec::new(); 9],
            battle_menu_texts: vec![Vec::new(); 0x24],
            battle_texts: vec![Vec::new(); 0x35],
            god_item_texts: vec![Default::default(); 4],
            god_item_names: vec![Vec::new(); 4],
            god_name_text: Default::default(),
            god_bought_texts: Default::default(),
            god_short_texts: Default::default(),
            god_chatter_texts: vec![Default::default(); 0x21],
            god_intro_texts: vec![Default::default(); 3],
            menu_texts: (0..0x3bc).map(|_| None).collect(),
            map_ui_sheet: Default::default(),
            map_reopen_times: Default::default(),
            item_drop_queue: Default::default(),
            drop_icons: BTreeMap::new(),
            web_popup_entries: Default::default(),
            web_popup_shown: Default::default(),
            item_snapshot: Default::default(),
            ad_button_id: Default::default(),
            ad_button_cleared: Default::default(),
            lineup_stages: Default::default(),
            lineup_records: Default::default(),
            labyrinth_units: Default::default(),
            labyrinth_floors: Default::default(),
            drop_items: Default::default(),
            stage_rewards_taken: Default::default(),
            event_reward_cache: Default::default(),
            clear_count_rewards: Default::default(),
            clear_lineups: Default::default(),
            map_clear_counts: Default::default(),
            medals_awarded: Default::default(),
            enigma_durations: Default::default(),
            enigma: Default::default(),
            map_open_neg4: Default::default(),
            map_open_neg9: Default::default(),
            map_open_neg10: Default::default(),
            map_open_neg11: Default::default(),
            map_open_neg16: Default::default(),
            map_open_neg17: Default::default(),
            map_open_neg18: Default::default(),
            map_open_neg20: Default::default(),
            map_open_neg22: Default::default(),
            map_open_neg23: Default::default(),
            map_open_neg24: Default::default(),
            map_open_neg26: Default::default(),
            xp_ad_maps: Default::default(),
            map_intervals: Default::default(),
            map_one_time: Default::default(),
            map_guerrilla_sets: Default::default(),
            aku_timers: Default::default(),
            altar_stage_values: Default::default(),
            stage_pair_progress: Default::default(),
            units_dirty: Default::default(),
            orb_inventory: Default::default(),
            stage_unlock_neg4: Default::default(),
            stage_unlock_neg9: Default::default(),
            stage_unlock_neg10: Default::default(),
            stage_unlock_neg11: Default::default(),
            stage_unlock_neg16: Default::default(),
            stage_unlock_neg17: Default::default(),
            stage_unlock_neg18: Default::default(),
            stage_unlock_neg19: Default::default(),
            stage_unlock_neg20: Default::default(),
            stage_unlock_neg22: Default::default(),
            stage_unlock_neg23: Default::default(),
            stage_unlock_neg24: Default::default(),
            stage_unlock_neg26: Default::default(),
            stage_unlock_cache: Default::default(),
            stages_cleared_neg26: Default::default(),
            stage_record_neg26: Default::default(),
            stage_record_neg24: Default::default(),
            stage_record_neg23: Default::default(),
            stage_record_neg22: Default::default(),
            stage_record_neg20: Default::default(),
            stage_record_neg19: Default::default(),
            stage_record_neg18: Default::default(),
            stage_record_neg17: Default::default(),
            stage_record_neg16: Default::default(),
            stage_record_neg11: Default::default(),
            stage_record_neg10: Default::default(),
            stage_record_neg9: Default::default(),
            stage_record_neg4: Default::default(),
            bgm_player_bound: false,
            demon_banner_anim: Default::default(),
            default_font: Vec::new(),
            combo_banner_texts: [None; 3],
            crit_vfx_anim: Default::default(),
            zkill_vfx_anim: Default::default(),
            barrier_anims: Default::default(),
            shield_anims: Default::default(),
            savage_vfx: Vec::new(),
            toxic_vfx: Vec::new(),
            metal_killer_vfx: Vec::new(),
            drain_vfx: Vec::new(),
            savage_vfx_anim: Default::default(),
            toxic_vfx_anim: Default::default(),
            metal_killer_vfx_anim: Default::default(),
            drain_vfx_anim: Default::default(),
            sound: None,
            text: None,
            platform: None,
            ui: None,
            meta: None,
            scene: None,
            assets: None,
        }
    }

    pub fn entity_field(faction: i32, slot: i32, field: usize) -> usize {
        (faction as usize)
            .wrapping_mul(FACTION_STRIDE)
            .wrapping_add((slot as usize).wrapping_mul(ENTITY_STRIDE))
            .wrapping_add(ENTITY_BASE)
            .wrapping_add(field)
    }

    pub fn cat_stat(unit_id: i32, form: i32, column: usize) -> usize {
        ((unit_id.wrapping_add(2) as i64) * CAT_STATS_UNIT_STRIDE as i64
            + (form as i64) * CAT_STATS_FORM_STRIDE as i64
            + CAT_STATS as i64
            + column as i64) as usize
    }

    pub fn enemy_stat(unit_id: i32, column: usize) -> usize {
        ((unit_id.wrapping_add(2) as i64) * ENEMY_STATS_STRIDE as i64
            + ENEMY_STATS as i64
            + column as i64) as usize
    }

    pub fn faction_flags(faction: i32) -> usize {
        (faction as usize)
            .wrapping_mul(FACTION_FLAGS_STRIDE)
            .wrapping_add(FACTION_FLAGS)
    }

    pub fn sound(&mut self) -> Option<&mut (dyn SoundManager + 'static)> {
        self.sound.as_deref_mut()
    }

    pub fn platform(&mut self) -> Option<&mut (dyn Platform + 'static)> {
        self.platform.as_deref_mut()
    }

    pub fn set_platform(&mut self, platform: Box<dyn Platform>) {
        self.platform = Some(platform);
    }

    pub fn text_renderer(&mut self) -> Option<&mut (dyn TextRenderer + 'static)> {
        self.text.as_deref_mut()
    }

    pub fn set_text_renderer(&mut self, text: Box<dyn TextRenderer>) {
        self.text = Some(text);
    }

    pub fn ui(&mut self) -> Option<&mut (dyn UiHost + 'static)> {
        self.ui.as_deref_mut()
    }

    pub fn meta(&mut self) -> Option<&mut (dyn MetaHost + 'static)> {
        self.meta.as_deref_mut()
    }

    pub fn set_meta(&mut self, meta: Box<dyn MetaHost>) {
        self.meta = Some(meta);
    }

    pub fn scene_host(&mut self) -> Option<&mut (dyn SceneHost + 'static)> {
        self.scene.as_deref_mut()
    }

    pub fn set_scene_host(&mut self, scene: Box<dyn SceneHost>) {
        self.scene = Some(scene);
    }

    pub fn assets(&mut self) -> Option<&mut (dyn AssetSource + 'static)> {
        self.assets.as_deref_mut()
    }

    pub fn set_assets(&mut self, assets: Box<dyn AssetSource>) {
        self.assets = Some(assets);
    }

    pub fn set_ui(&mut self, ui: Box<dyn UiHost>) {
        self.ui = Some(ui);
    }

    pub fn set_sound(&mut self, sound: Box<dyn SoundManager>) {
        self.sound = Some(sound);
    }

    pub fn rng_state(&self) -> u32 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&self.raw[RNG_STATE..RNG_STATE + 4]);

        u32::from_le_bytes(word)
    }

    pub fn set_rng_state(&mut self, state: u32) {
        self.raw[RNG_STATE..RNG_STATE + 4].copy_from_slice(&state.to_le_bytes());
    }

    pub fn zero(&mut self, off: usize, len: usize) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..len))
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: off as i64,
                limit: SIZE as i64,
            })?;

        bytes.fill(0);

        Ok(())
    }

    pub fn block_at<const N: usize>(&self, off: usize) -> Result<[u8; N], Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..N))
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: off as i64,
                    limit: SIZE as i64,
                })?;

        let mut block = [0u8; N];
        block.copy_from_slice(bytes);

        Ok(block)
    }

    pub fn set_block_at<const N: usize>(
        &mut self,
        off: usize,
        value: [u8; N],
    ) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..N))
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: off as i64,
                limit: SIZE as i64,
            })?;

        bytes.copy_from_slice(&value);

        Ok(())
    }

    pub fn bytes_from(&self, off: usize) -> Result<&[u8], Fault> {
        self.raw.get(off..).ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: off as i64,
            limit: SIZE as i64,
        })
    }

    pub fn f32_at(&self, off: usize) -> Result<f32, Fault> {
        Ok(f32::from_le_bytes(self.block_at::<4>(off)?))
    }

    pub fn set_f32_at(&mut self, off: usize, value: f32) -> Result<(), Fault> {
        self.set_block_at::<4>(off, value.to_le_bytes())
    }

    pub fn u8_at(&self, off: usize) -> Result<u8, Fault> {
        self.raw.get(off).copied().ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: off as i64,
            limit: SIZE as i64,
        })
    }

    pub fn i8_at(&self, off: usize) -> Result<i8, Fault> {
        self.raw
            .get(off)
            .map(|byte| *byte as i8)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: off as i64,
                limit: SIZE as i64,
            })
    }

    pub fn i16_at(&self, off: usize) -> Result<i16, Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..2))
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: off as i64,
                    limit: SIZE as i64,
                })?;

        let mut word = [0u8; 2];
        word.copy_from_slice(bytes);

        Ok(i16::from_le_bytes(word))
    }

    pub fn i32_at(&self, off: usize) -> Result<i32, Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..4))
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: off as i64,
                    limit: SIZE as i64,
                })?;

        let mut word = [0u8; 4];
        word.copy_from_slice(bytes);

        Ok(i32::from_le_bytes(word))
    }

    pub fn set_i32_at(&mut self, off: usize, value: i32) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..4))
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: off as i64,
                limit: SIZE as i64,
            })?;

        bytes.copy_from_slice(&value.to_le_bytes());

        Ok(())
    }
}
