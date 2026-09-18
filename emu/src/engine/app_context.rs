use std::collections::BTreeMap;

use crate::Fault;

use super::{CastleRow, CharaGroup, ComboStore, FixedLineupStore, MapData, OrbStore, SoundManager, SpecialRuleStore, TreasureStore};

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
    pub const KEY: usize = 0xfc;
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
    pub const CANNON_READY_FX: usize = 0x74;
    pub const CANNON_NONMETAL_PERMILLE: usize = 0x78;
    pub const CANNON_NONZOMBIE_PERMILLE: usize = 0x78;
    pub const CANNON_BURROWED_PERMILLE: usize = 0x7c;
}

pub struct Entity;

impl Entity {
    pub const OCCUPANT: usize = 0x0;
    pub const STATE: usize = 0x4;
    pub const FRAME: usize = 0x8;
    pub const POS_X: usize = 0xc;
    pub const POS_Y: usize = 0x10;
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
    pub const CRIT_FX: usize = 0x98;
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
    pub const WEAKEN_ACTIVE_PCT: usize = 0x13c;
    pub const ATTACK_1_LD_ANCHOR: usize = 0x150;
    pub const WAVE_IMMUNE: usize = 0x158;
    pub const WAVE_BLOCK: usize = 0x15c;
    pub const KNOCKBACK_IMMUNE: usize = 0x160;
    pub const FREEZE_IMMUNE: usize = 0x164;
    pub const SLOW_IMMUNE: usize = 0x168;
    pub const WEAKEN_IMMUNE: usize = 0x16c;
    pub const BURROW_COUNT: usize = 0x184;
    pub const BURROW_START_X: usize = 0x188;
    pub const IMMUNE_FX_FRAME: usize = 0x194;
    pub const IMMUNE_FX_ACTIVE: usize = 0x198;
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
    pub const BARRIER_FX_ACTIVE: usize = 0x214;
    pub const BARRIER_FX_FRAME: usize = 0x218;
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
    pub const SAVAGE_BLOW_FX: usize = 0x298;
    pub const DODGE_TIMER: usize = 0x29c;
    pub const DODGE_DURATION: usize = 0x2a0;
    pub const DODGE_FX_FRAME: usize = 0x2a4;
    pub const TOXIC_CHANCE: usize = 0x2ac;
    pub const TOXIC_DAMAGE: usize = 0x2b0;
    pub const TOXIC_FX: usize = 0x2b4;
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
    pub const SHIELD_FX: usize = 0x300;
    pub const SHIELD_FX_FRAME: usize = 0x304;
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
    pub const METAL_KILLER_FX: usize = 0x36c;
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
    sound: Option<Box<dyn SoundManager>>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    pub const DECK_KEY: usize = 0x28;
    pub const DECK_STRIDE: usize = 0x2c;
    pub const WALLET_DEPLOY_COUNTS: usize = 0x108;
    pub const EX_REDIRECT_A_BLOCKED: usize = 0x1490;
    pub const SCENE_ID: usize = 0x3450;
    pub const DECK_PRESETS: usize = 0xc310;
    pub const FACTION_1_DECK: usize = 0xc33c;
    pub const BATTLE_DECK: usize = 0xc6ac;
    pub const STAGES_CLEARED_CHAPTERS: usize = 0xc94c;
    pub const STAGE_RECORD_CHAPTERS: usize = 0xc978;
    pub const FACTION_1_UNIT_FORMS: usize = 0x495f4;
    pub const STAGE_CASTLE_ID: usize = 0x836fc;
    pub const TREASURE_PROGRESS: usize = 0x83708;
    pub const STAGE_LENGTH: usize = 0x9e528;
    pub const STAGE_INDEX: usize = 0x325c48;
    pub const CHAPTER_MODE: usize = 0x327efc;
    pub const FACTION_1_BUTTON_ROWS: usize = 0x327f18;
    pub const BUTTON_UNIT_FORMS: usize = 0x327f40;
    pub const BATTLE_IS_OUTBREAK: usize = 0x32c5c8;
    pub const OUTBREAKS_ENABLED: usize = 0x32c5c9;
    pub const BATTLE_IS_INVASION: usize = 0x32c5d6;
    pub const BATTLE_IS_Z_INVASION: usize = 0x32c5d7;
    pub const INVASION_STAGE: usize = 0x32c5d8;
    pub const MAP_NEG15_CLEARED: usize = 0x32c5d9;
    pub const MAP_NEG25_CLEARED: usize = 0x32c5da;
    pub const MAP_INDEX: usize = 0x3388b8;
    pub const STAGES_CLEARED_STORY: usize = 0x33e020;
    pub const STAGES_CLEARED_NEG6: usize = 0x340748;
    pub const STAGE_RECORD_STORY: usize = 0x340818;
    pub const STAGE_RECORD_NEG6: usize = 0x37b1b0;
    pub const SAVED_MAP_TYPE: usize = 0x3836b4;
    pub const CHAPTER_COST_TIER: usize = 0x388028;
    pub const SELECTED_DECK_PRESET: usize = 0x38fd6c;
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

    pub fn new() -> Self {
        Self {
            raw: vec![0u8; SIZE].into_boxed_slice(),
            stage_enemies: Vec::new(),
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
            sound: None,
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
        ((unit_id.wrapping_add(2) as i64) * ENEMY_STATS_STRIDE as i64 + ENEMY_STATS as i64 + column as i64) as usize
    }

    pub fn faction_flags(faction: i32) -> usize {
        (faction as usize).wrapping_mul(FACTION_FLAGS_STRIDE).wrapping_add(FACTION_FLAGS)
    }

    pub fn sound(&mut self) -> Option<&mut (dyn SoundManager + 'static)> {
        self.sound.as_deref_mut()
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
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.fill(0);

        Ok(())
    }

    pub fn block_at<const N: usize>(&self, off: usize) -> Result<[u8; N], Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..N))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut block = [0u8; N];
        block.copy_from_slice(bytes);

        Ok(block)
    }

    pub fn set_block_at<const N: usize>(&mut self, off: usize, value: [u8; N]) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..N))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.copy_from_slice(&value);

        Ok(())
    }

    pub fn bytes_from(&self, off: usize) -> Result<&[u8], Fault> {
        self.raw.get(off..).ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn u8_at(&self, off: usize) -> Result<u8, Fault> {
        self.raw
            .get(off)
            .copied()
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn i8_at(&self, off: usize) -> Result<i8, Fault> {
        self.raw
            .get(off)
            .map(|byte| *byte as i8)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn i16_at(&self, off: usize) -> Result<i16, Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..2))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut word = [0u8; 2];
        word.copy_from_slice(bytes);

        Ok(i16::from_le_bytes(word))
    }

    pub fn i32_at(&self, off: usize) -> Result<i32, Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..4))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut word = [0u8; 4];
        word.copy_from_slice(bytes);

        Ok(i32::from_le_bytes(word))
    }

    pub fn set_i32_at(&mut self, off: usize, value: i32) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..4))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.copy_from_slice(&value.to_le_bytes());

        Ok(())
    }
}
