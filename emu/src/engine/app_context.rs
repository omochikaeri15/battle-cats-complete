use std::{
    cell,
    collections::{BTreeMap, BTreeSet},
    rc::{Rc, Weak},
};

use crate::{Entropy, Fault};

use super::{
    AdRewardRow, AltarReward, AssetSource, BaseShake, BattleEventLatch, BgEffects,
    BuiltDeckRecord, ButtonBank, CannonGrowthStep, CannonPart, CastleRecipeEntry, CastleRecipeUnlockRow, CastleRow, CatseyeStep, ChangeCondition, CharaGroup, ComboStore,
    CounterSurgeEvent, DailyLoginGrade, DialogManager, DojoChestRow, DrawSink, EffectSprite, Enigma, EventGatyaGroup, EventItemStore,
    ExGroup, ExplosionEvent, FixedLineupStore, GatyaChanceAnimation, GatyaDataSet, HiddenData, GamatotoBonus, GamatotoCollabo, GamatotoSpecialDrop, Imgcut, ItemPackRow, ItemShopRow, LabyrinthFloor, LineupRecord, Maanim,
    Mamodel, MapLayout, Medal, MapOption, OfficersClubRow, OrbEffectStore, MapRecord, MapStageShortcut, MatatabiRow, MetaHost, OrbStore, Platform, RankingRecord, ReleasePoint,
    DropItemRow, EventDisplayRow, MissionConditionSetting, MissionData, MissionGatyaSetting, MissionLimitOption, MissionMonthly,
    PointEventReward, RealmsRngTable, RecommendedLevelup, SceneHost, ScreenMetrics, SheetTable, SoundManager, SoundState, SpecialRuleStore,
    StagePairRecord, StageRestriction, SurgeEvent, TextBlock, TextRenderer, Texture, TreasureGauge, TreasureStore,
    UiHost, UnlockGroup, VibrationStore, UnlockPopupRow, WebPopupEntry, ZombieLotteryRow,
};

const GAME_SIZE: usize = 0x47c170;
const EXACT_LAYOUT: bool = cfg!(feature = "exact-layout");
const DROP_MAP_CELLS: usize = 48;

pub const UNITS: usize = 0x372;
pub const ENEMY_ROWS: usize = 0x324;
pub const TALENT_GROUPS: usize = 0xb;
pub const EX_MAPS: usize = 0x52;
pub const NEG5_MAPS: usize = 0x10;
pub const LABEL_SPARES: usize = 0xc8;
pub const UNIT_CAPACITY: usize = if EXACT_LAYOUT { UNITS } else { 0x800 };
pub const ENEMY_ROW_CAPACITY: usize = if EXACT_LAYOUT { ENEMY_ROWS } else { 0x800 };

pub const ENTITY_BASE: usize = 0x84080;
pub const ENTITY_STRIDE: usize = 0x3ec;
pub const FACTION_STRIDE: usize = 0xc804;
pub const SLOTS_PER_FACTION: i32 = 51;

pub const STAGE_ENEMY_COLUMNS: usize = 14;

pub const UNIT_BUY_STRIDE: usize = 0x100;
pub const CAT_STATS_UNIT_STRIDE: usize = 0x770;
pub const CAT_STATS_FORM_STRIDE: usize = 0x1dc;
pub const ENEMY_STATS_STRIDE: usize = 0x1c4;

const TAIL_UNIT_BUY: usize = GAME_SIZE;
const TAIL_CAT_STATS: usize = TAIL_UNIT_BUY + UNIT_CAPACITY * UNIT_BUY_STRIDE;
const TAIL_ENEMY_STATS: usize = TAIL_CAT_STATS + (UNIT_CAPACITY + 2) * CAT_STATS_UNIT_STRIDE;
const TAIL_UNITS_OWNED: usize = TAIL_ENEMY_STATS + ENEMY_ROW_CAPACITY * ENEMY_STATS_STRIDE;
const TAIL_UNIT_LEVELS: usize = TAIL_UNITS_OWNED + UNIT_CAPACITY * 4 + 4;
const TAIL_UNIT_FORMS: usize = TAIL_UNIT_LEVELS + UNIT_CAPACITY * 8;
const TAIL_REWARD_UNITS_OWNED: usize = TAIL_UNIT_FORMS + UNIT_CAPACITY * 4;
const TAIL_REWARD_FORMS_OWNED: usize = TAIL_REWARD_UNITS_OWNED + UNIT_CAPACITY * 4;
const TAIL_UNIT_FORM_COUNTS: usize = TAIL_REWARD_FORMS_OWNED + UNIT_CAPACITY * 4;
const TAIL_UNIT_LEVEL_CURVE: usize = TAIL_UNIT_FORM_COUNTS + UNIT_CAPACITY * 0x20;
const TAIL_UNIT_EXP_CURVE: usize = TAIL_UNIT_LEVEL_CURVE + UNIT_CAPACITY * 0x50;
const TAIL_SEEN_ENEMIES: usize = TAIL_UNIT_EXP_CURVE + UNIT_CAPACITY * 0x50;
const TAIL_END: usize = TAIL_SEEN_ENEMIES + ENEMY_ROW_CAPACITY * 4;

pub const SIZE: usize = if EXACT_LAYOUT { GAME_SIZE } else { TAIL_END };

const fn moved(game: usize, tail: usize) -> usize {
    if EXACT_LAYOUT { game } else { tail }
}

pub const UNIT_BUY: usize = moved(0x4b060, TAIL_UNIT_BUY);
pub const CAT_STATS: usize = moved(0x9ee88, TAIL_CAT_STATS);
pub const ENEMY_STATS: usize = moved(0x239da8, TAIL_ENEMY_STATS);

const FACTION_FLAGS: usize = 0x2678;
const FACTION_FLAGS_STRIDE: usize = 0x1f0;

const _: () = assert!(ENTITY_BASE + 2 * FACTION_STRIDE <= 0x9ee88);
const _: () = assert!(UNITS <= UNIT_CAPACITY && ENEMY_ROWS <= ENEMY_ROW_CAPACITY);

pub struct Limits {
    pub units: i32,
    pub enemy_rows: i32,
    pub talent_groups: i32,
    pub ex_maps: i32,
    pub neg5_maps: i32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            units: UNITS as i32,
            enemy_rows: ENEMY_ROWS as i32,
            talent_groups: TALENT_GROUPS as i32,
            ex_maps: EX_MAPS as i32,
            neg5_maps: NEG5_MAPS as i32,
        }
    }
}

impl Limits {
    pub fn talent_row(&self) -> Vec<i32> {
        vec![0; (self.talent_groups as usize).wrapping_mul(0xe).wrapping_add(1)]
    }
}

pub struct UnitBuy;

impl UnitBuy {
    pub const RARITY: usize = 0x34;
    pub const GUIDE_ORDER: usize = 0x38;
    pub const TRUE_FORM_LEVEL: usize = 0x50;
    pub const MAX_LEVEL: usize = 0xc8;
    pub const MAX_PLUS_LEVEL: usize = 0xcc;
    pub const AVAILABLE: usize = 0xe4;
    pub const AVAILABLE_SIGN: usize = 0xe7;
    pub const ALT_ART: usize = 0xf4;
    pub const KEY: usize = 0xfc;
    pub const KEY_SIGN: usize = 0xff;
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

pub struct GatyaItem;

impl GatyaItem {
    pub const RARITY: usize = 0x0;
    pub const PRICE: usize = 0x8;
    pub const STAGE_DROP_ITEM_ID: usize = 0x10;
    pub const QUANTITY: usize = 0x18;
    pub const SERVER_ID: usize = 0x20;
    pub const CATEGORY: usize = 0x24;
    pub const INDEX: usize = 0x28;
    pub const SRC_ITEM_ID: usize = 0x2c;
    pub const MAIN_MENU_TYPE: usize = 0x30;
    pub const GATYA_TICKET_ID: usize = 0x34;
    pub const IMG_ID: usize = 0x38;
    pub const REFLECT_OR_STORAGE: usize = 0x3c;
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
    pub const PROC_EXTRA: usize = 0x24;
    pub const MINI: usize = 0x2c;
}

pub struct OptionPage;

impl OptionPage {
    pub const LEFT: usize = 0x10;
    pub const RIGHT: usize = 0x14;
    pub const TOP: usize = 0x18;
    pub const BOTTOM: usize = 0x1c;
    pub const SIDE: usize = 0x20;
    pub const HEADER: usize = 0x24;
    pub const FOOTER: usize = 0x28;
    pub const TALL: usize = 0x2c;
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
    pub const FIRST_ID: usize = 0x14;
    pub const SECOND_ID: usize = 0x18;
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
    pub const STRIDE: usize = 0xc;
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

pub struct Rect;

impl Rect {
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const WIDTH: usize = 0x8;
    pub const HEIGHT: usize = 0xc;
    pub const STRIDE: usize = 0x10;
}

pub struct Vector;

impl Vector {
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
}

pub struct Matrix;

impl Matrix {
    pub const RIGHT_X: usize = 0x0;
    pub const RIGHT_Y: usize = 0x4;
    pub const DOWN_X: usize = 0x8;
    pub const DOWN_Y: usize = 0xc;
}

pub struct Quad;

impl Quad {
    pub const CORNER_0: usize = 0x0;
    pub const CORNER_1: usize = 0x4;
    pub const CORNER_2: usize = 0x8;
    pub const CORNER_3: usize = 0xc;
}

pub struct Cells;

impl Cells {
    pub const FIRST: usize = 0x0;
    pub const SECOND: usize = 0x4;
    pub const THIRD: usize = 0x8;
}

pub struct ProcRolls;

impl ProcRolls {
    pub const CRITICAL: usize = 0x0;
    pub const KNOCKBACK: usize = 0x4;
    pub const FREEZE: usize = 0x8;
    pub const SLOW: usize = 0xc;
    pub const WEAKEN: usize = 0x10;
    pub const SAVAGE_BLOW: usize = 0x14;
    pub const WARP: usize = 0x18;
    pub const BARRIER_BREAKER: usize = 0x1c;
    pub const CURSE: usize = 0x20;
    pub const TOXIC: usize = 0x24;
    pub const SHIELD_PIERCE: usize = 0x28;
    pub const DRAIN: usize = 0x2c;
    pub const SIZE: usize = 0x30;
}

pub struct BgSetup;

impl BgSetup {
    pub const SKY_TOP: usize = 0x0;
    pub const SKY_BOTTOM: usize = 0x4;
    pub const GROUND_TOP: usize = 0x8;
    pub const GROUND_BOTTOM: usize = 0xc;
    pub const MODEL_ID: usize = 0x10;
    pub const HAS_UPPER_LAYER: usize = 0x14;
    pub const IMAGE_ID: usize = 0x18;
    pub const GRADIENT_TOP: usize = 0x1c;
    pub const GRADIENT_BOTTOM: usize = 0x20;
}

pub struct BgParticle;

impl BgParticle {
    pub const STRIDE: usize = 0x14;
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const PHASE: usize = 0x8;
    pub const SPEED: usize = 0xc;
    pub const KIND: usize = 0x10;
}

pub struct BgDrifter;

impl BgDrifter {
    pub const STRIDE: usize = 0x10;
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const ANGLE: usize = 0x8;
    pub const RADIUS: usize = 0xc;
}

pub struct BgStar;

impl BgStar {
    pub const STRIDE: usize = 0x10;
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const GLOW: usize = 0x8;
}

pub struct BgSprite;

impl BgSprite {
    pub const STRIDE: usize = 0x40;
    pub const HALF: usize = 0x20;
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const ANGLE: usize = 0x8;
    pub const SIZE: usize = 0xc;
    pub const RATE: usize = 0x10;
    pub const LIFE: usize = 0x14;
    pub const TURN: usize = 0x18;
}

pub struct SniperCasing;

impl SniperCasing {
    pub const STRIDE: usize = 0x14;
    pub const X: usize = 0x0;
    pub const Y: usize = 0x4;
    pub const AGE: usize = 0x8;
    pub const DRIFT_X: usize = 0xc;
    pub const DRIFT_Y: usize = 0x10;
}

pub struct StrikeSparks;

impl StrikeSparks {
    pub const FIRST_TIMER: usize = 0x0;
    pub const FIRST_X: usize = 0x4;
    pub const FIRST_Y: usize = 0x8;
    pub const SECOND_TIMER: usize = 0xc;
    pub const SECOND_X: usize = 0x10;
    pub const SECOND_Y: usize = 0x14;
}

pub struct DrawEntry;

impl DrawEntry {
    pub const STRIDE: usize = 0xc;
    pub const Z: usize = 0x0;
    pub const ORDER: usize = 0x4;
    pub const SLOT: usize = 0x8;
}

pub struct HitEntry;

impl HitEntry {
    pub const STRIDE: usize = 0x8;
    pub const SLOT: usize = 0x0;
    pub const DISTANCE: usize = 0x4;
}

pub struct MapStageRow;

impl MapStageRow {
    pub const STRIDE: usize = 0xbc;
    pub const MUSIC: usize = 0x8;
    pub const MUSIC_SWITCH: usize = 0xc;
    pub const BOSS_MUSIC: usize = 0x10;
    pub const KEY: usize = 0xb8;
}

pub struct StageRecordRow;

impl StageRecordRow {
    pub const STRIDE: usize = 0xd0;
    pub const KEY: usize = 0xcc;
}

pub struct TreasureRow;

impl TreasureRow {
    pub const KEY: usize = 0xc4;
}

pub struct TechMax;

impl TechMax {
    pub const LEVEL: usize = 0x0;
    pub const PLUS: usize = 0x4;
}

pub struct StampEntry;

impl StampEntry {
    pub const STRIDE: usize = 0x8;
    pub const FIRST: usize = 0x0;
    pub const SECOND: usize = 0x4;
}

pub struct PowerupGrant;

impl PowerupGrant {
    pub const STRIDE: usize = 0x18;
    pub const GRANTED: usize = 0x1;
}

pub struct Labyrinth;

impl Labyrinth {
    pub const FLOORS: usize = 0x18;
    pub const CLEARED_COUNT: usize = 0x338;
    pub const MAP_ID: usize = 0x550;
}

pub struct FloorCell;

impl FloorCell {
    pub const STRIDE: usize = 0x8;
    pub const FIRST: usize = 0x0;
    pub const SECOND: usize = 0x4;
}

pub struct BaseGuardNotice;

impl BaseGuardNotice {
    pub const STATE: usize = 0x0;
    pub const FRAME: usize = 0x4;
}

pub struct CatGodButton;

impl CatGodButton {
    pub const INTRO: usize = 0x3;
    pub const CLOSE: usize = 0x4;
    pub const CONFIRM: usize = 0x5;
    pub const BACK: usize = 0x6;
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
    pub const RECHARGE_CUT: usize = 0x3e8;
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
    pub const RECHARGE_CUT: usize = 0x1d8;
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

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ScoreBonusMap {
    pub name_label: Vec<u8>,
    pub explanation_label: Vec<u8>,
    pub bonuses: BTreeMap<i32, Vec<i32>>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StageNameTable {
    pub names: Vec<Vec<Vec<u8>>>,
    pub stage_unlock: Vec<Vec<i8>>,
    pub stages_cleared: Vec<Vec<i8>>,
    pub stage_record: Vec<Vec<Vec<i16>>>,
    pub map_open: Vec<Vec<i8>>,
    pub map_flag: Vec<u8>,
    pub map_opened: Vec<bool>,
    pub map_stamp: Vec<i64>,
    pub map_stamp_alt: Vec<i64>,
}

pub struct AppContext {
    raw: Box<[u8]>,
    rng_state: u32,
    pub limits: Limits,
    pub treasure_gauge: TreasureGauge,
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
    pub cleared_session_keys: Vec<i32>,
    pub stage_conditions: BTreeMap<i32, BTreeMap<i32, BTreeMap<i32, i32>>>,
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
    pub ex_groups: BTreeMap<i32, ExGroup>,
    pub ex_lottery: Vec<[i32; 2]>,
    pub hidden_drop_keys: BTreeMap<i32, Vec<u8>>,
    pub stage_pair_records: BTreeMap<i32, StagePairRecord>,
    pub altar_enemy_ids: BTreeMap<i32, i32>,
    pub altar_rewards: BTreeMap<i32, AltarReward>,
    pub label_texts: Vec<Option<Texture>>,
    pub map_names: BTreeMap<i32, Vec<u8>>,
    pub drop_item_rows: BTreeMap<i32, DropItemRow>,
    pub filter_stage_maps: Vec<i32>,
    pub slot_unlock_rows: [[i32; 2]; 19],
    pub matatabi_rows: Vec<MatatabiRow>,
    pub item_shop_rows: BTreeMap<i32, ItemShopRow>,
    pub rank_gift_messages: Vec<Vec<u8>>,
    pub warning1_texts: Vec<Vec<u8>>,
    pub warning2_rows: Vec<[Vec<u8>; 4]>,
    pub main_menu_rows: Vec<Vec<Vec<u8>>>,
    pub main_menu_row_settings: Vec<i32>,
    pub main_menu_text_settings: Vec<Vec<i32>>,
    pub god_explanation: [Vec<u8>; 4],
    pub stage_first_messages: Vec<[Vec<u8>; 4]>,
    pub challenge_mode_texts: [[Vec<u8>; 4]; 3],
    pub page_names: Vec<Vec<u8>>,
    pub first_lose_texts: [Vec<u8>; 2],
    pub option_rows: [[Vec<u8>; 3]; 3],
    pub main_menu_popups: Vec<[Vec<u8>; 4]>,
    pub tutorial_pages: Vec<[Vec<u8>; 12]>,
    pub popup_messages: Vec<[Vec<u8>; 10]>,
    pub category_explanations: Vec<[Vec<u8>; 4]>,
    pub stamp_messages: Vec<Vec<u8>>,
    pub gift_messages: Vec<[Vec<u8>; 4]>,
    pub unit_evolve_rows: Vec<[Vec<u8>; 6]>,
    pub matatabi_popups: Vec<[Vec<u8>; 4]>,
    pub device_screen_w: i32,
    pub device_screen_h: i32,
    pub event_display: BTreeMap<i32, EventDisplayRow>,
    pub daily_login_rows: Vec<Vec<i32>>,
    pub daily_login_groups: Vec<Vec<Vec<i32>>>,
    pub daily_login_max_id: i32,
    pub beacon_rows: Vec<Vec<i32>>,
    pub beacon_groups: Vec<Vec<Vec<i32>>>,
    pub reward_stage_ids: Vec<i32>,

    pub lose_rows: Vec<Vec<Vec<u8>>>,
    pub lose_row_settings: Vec<i32>,
    pub lose_text_settings: Vec<Vec<i32>>,
    pub web_popup_pending: BTreeMap<i32, bool>,
    pub img004_sheet: Option<Rc<Imgcut>>,
    pub reward_claimed: BTreeMap<i32, Vec<i32>>,
    pub release_points: BTreeMap<i32, ReleasePoint>,
    pub server_flags: BTreeMap<i32, bool>,
    pub drop_chara_max_1000: i32,
    pub drop_chara_max_1100: i32,
    pub img039_sheet: Option<Rc<Imgcut>>,
    pub deck_button_x: [i32; 10],
    pub enemy_kill_counts: BTreeMap<i32, i32>,
    pub best_scores: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub ranking_entries: Vec<Option<RankingRecord>>,
    pub boss_shockwave_anim: Maanim,
    pub cannon_part_rows: BTreeMap<i32, Vec<i32>>,
    pub ex_option_targets: BTreeMap<i32, i32>,
    pub ex_replacement_stages: BTreeMap<i32, Vec<i32>>,
    pub built_deck_stages: BTreeMap<i16, Vec<i32>>,
    pub built_deck_records: BTreeMap<i16, BuiltDeckRecord>,
    pub cannon_parts: BTreeMap<i32, CannonPart>,
    pub enemy_castle: Vec<CastleRow>,
    pub fixed_lineup_store: FixedLineupStore,
    pub combo_store: ComboStore,
    pub chara_groups: BTreeMap<i32, CharaGroup>,
    pub crown_multipliers: BTreeMap<i32, Vec<i32>>,
    pub map_options: MapOption,
    pub settings: BTreeMap<Vec<u8>, Vec<u8>>,
    pub failed_packs: BTreeSet<Vec<u8>>,
    pub entropy: Entropy,
    pub dojo_chest_rows: Vec<DojoChestRow>,
    pub gold_cpu_rows: Vec<[i32; 4]>,
    pub lock_skip_rows: Vec<[i32; 2]>,
    pub realms_rng_tables: BTreeMap<i32, RealmsRngTable>,
    pub point_event_rewards: BTreeMap<i32, PointEventReward>,
    pub event_gatya_items: BTreeMap<i32, EventGatyaGroup>,
    pub leadership_return_maps: BTreeMap<i32, Vec<i32>>,
    pub event_gatya_settings: BTreeMap<i32, BTreeMap<i32, BTreeMap<i32, i32>>>,
    pub orb_effects: [OrbEffectStore; 2],
    pub catseye_behavior: BTreeMap<i32, BTreeMap<i32, Vec<CatseyeStep>>>,
    pub ability_data_rows: [[i32; 5]; 10],
    pub gatya_data_sets: BTreeMap<i32, Vec<GatyaDataSet>>,
    pub gatya_chance_animations: Vec<GatyaChanceAnimation>,
    pub recommended_powerup_rows: Vec<[i32; 4]>,
    pub recommended_levelups: BTreeMap<i32, RecommendedLevelup>,
    pub officers_club_rows: BTreeMap<i32, OfficersClubRow>,
    pub item_pack_rows: BTreeMap<i32, ItemPackRow>,
    pub autoset_ratings: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub autoset_abilities: Vec<[i32; 4]>,
    pub autoset_excluded_enemies: Vec<i32>,
    pub autoset_groups: BTreeMap<i32, i32>,
    pub autoset_organizations: BTreeMap<i32, [[i32; 3]; 10]>,
    pub map_layouts: BTreeMap<i32, MapLayout>,
    pub vibration: VibrationStore,
    pub daily_login_grades: BTreeMap<i32, DailyLoginGrade>,
    pub sound_state: SoundState,
    pub change_conditions: BTreeMap<i32, ChangeCondition>,
    pub score_bonus_maps: BTreeMap<i32, ScoreBonusMap>,
    pub hidden_data: HiddenData,
    pub medals: Vec<Medal>,
    pub medal_order: Vec<u64>,
    pub treasure_store: TreasureStore,
    pub orb_store: OrbStore,
    pub special_rules: SpecialRuleStore,
    pub equipped_orbs: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub map_data_ids: BTreeMap<i32, Vec<i32>>,
    pub map_stage_sets: BTreeMap<i32, Vec<i32>>,
    pub talent_definitions: BTreeMap<i32, Vec<i32>>,
    pub talent_levels: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub gamatoto_logs: [Vec<Vec<u8>>; 3],
    pub gamatoto_log_counts: [i32; 3],
    pub gamatoto_member_stats: Vec<[i32; 3]>,
    pub gamatoto_member_names: Vec<[Vec<u8>; 4]>,
    pub gamatoto_messages: Vec<[Vec<u8>; 5]>,
    pub gamatoto_stage_free: Vec<[Vec<u8>; 3]>,
    pub gamatoto_stage_names: Vec<Vec<u8>>,
    pub gamatoto_stage_event_names: Vec<Vec<u8>>,
    pub gamatoto_limit: [i32; 3],
    pub gamatoto_stage_rows: Vec<[i32; 45]>,
    pub gamatoto_stage_event_rows: Vec<[i32; 45]>,
    pub gamatoto_drop_rows: Vec<[i32; 4]>,
    pub gamatoto_member_ids: [i32; 71],
    pub ad_reward_rows: Vec<AdRewardRow>,
    pub gamatoto_collabo: BTreeMap<i32, GamatotoCollabo>,
    pub gamatoto_collabo_stages: Vec<i32>,
    pub mission_gatya_settings: BTreeMap<i32, Rc<MissionGatyaSetting>>,
    pub mission_limit_options: BTreeMap<i32, Rc<MissionLimitOption>>,
    pub mission_unlock_conditions: BTreeMap<i32, [i32; 10]>,
    pub mission_data: BTreeMap<i32, MissionData>,
    pub mission_names: BTreeMap<i32, Vec<u8>>,
    pub mission_descriptions: BTreeMap<i32, Vec<u8>>,
    pub mission_monthly: BTreeMap<i32, MissionMonthly>,
    pub mission_groups: BTreeMap<i32, Vec<i32>>,
    pub mission_condition_settings: BTreeMap<i32, Rc<MissionConditionSetting>>,
    pub mission_max_type1: i32,
    pub mission_max_type3: i32,
    pub mission_max_type2: i32,
    pub mission_max_type0: i32,
    pub map_stage_limit_messages: BTreeMap<i32, [Vec<u8>; 3]>,
    pub stage_names_numbered: [Vec<Vec<u8>>; 3],
    pub stage_name_variants: [Vec<Vec<Vec<u8>>>; 17],
    pub stage_difficulty: BTreeMap<i32, Vec<f32>>,
    pub treasure1_texts: [Vec<Vec<u8>>; 10],
    pub treasure2_texts: [Vec<u8>; 13],
    pub treasure3_texts: [[[Vec<u8>; 3]; 23]; 10],
    pub treasure3_after_texts: [[Vec<u8>; 3]; 23],
    pub castle_recipe_unlock_data: BTreeMap<i32, CastleRecipeUnlockRow>,
    pub castle_recipe_unlocks: BTreeMap<i32, Vec<Vec<i32>>>,
    pub castle_mix_recipes: BTreeMap<i32, [i32; 5]>,
    pub castle_recipe_names: BTreeMap<i32, Vec<u8>>,
    pub castle_recipe_texts: BTreeMap<i32, Vec<Vec<u8>>>,
    pub castle_recipes: BTreeMap<i32, BTreeMap<i32, CastleRecipeEntry>>,
    pub castle_custom_limit: i32,
    pub gamatoto_bonus: GamatotoBonus,
    pub gamatoto_special_drops: BTreeMap<i32, GamatotoSpecialDrop>,
    pub gamatoto_unlocks: BTreeMap<i32, [i32; 3]>,
    pub ending_messages: Vec<[Vec<u8>; 4]>,
    pub opening_messages: [Vec<Vec<u8>>; 4],
    pub map_stage_shortcuts: [BTreeMap<i32, Vec<MapStageShortcut>>; 2],
    pub drop_chara_rows: Vec<Vec<i32>>,
    pub rank_gift_rows: Vec<[i32; 21]>,
    pub unit_limit_rows: Vec<[i32; 10]>,
    pub unit_limit_extra: [i32; 110],
    pub zombie_lottery: BTreeMap<i32, ZombieLotteryRow>,
    pub unlock_popup_rows: BTreeMap<i32, UnlockPopupRow>,
    pub outbreak_active: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub outbreak_cleared: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub cleared_map_ids: Vec<i32>,
    pub stages_cleared_cache: BTreeMap<i32, [i16; 4]>,
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
    pub dialog_sheet: Option<Rc<Imgcut>>,
    pub scene_img008_sheet: Option<Rc<Imgcut>>,
    pub guide_label: Texture,
    pub tutorial_lines: [Option<Texture>; 8],
    pub surface_lost: bool,
    pub ui_sheet_cache: [Option<Rc<Imgcut>>; 0x16],
    pub medals_awarded_flags: BTreeMap<i32, i32>,
    pub medals_pending: Vec<i32>,
    pub club_user_rank: i32,
    pub club_pass_state: [u8; 8],
    pub club_owned: BTreeMap<i32, i32>,
    pub trait_icons: BTreeMap<i32, bool>,
    pub ability_icons: BTreeMap<i32, bool>,
    pub enemy_book_rows: Vec<[Vec<u8>; 5]>,
    pub enemy_book_pages: Vec<[Vec<u8>; 12]>,
    pub enemy_book_question: [Vec<u8>; 3],
    pub cat_book_pages: Vec<[Vec<u8>; 12]>,
    pub cat_book_question: [Vec<u8>; 3],
    pub cat_book_rows: Vec<[Vec<u8>; 5]>,
    pub cat_book_data: Vec<[i32; 8]>,
    pub enemy_dictionary_ids: Vec<i32>,
    pub enemy_dictionary_pages: Vec<i32>,
    pub enemy_dictionary_groups: Vec<i32>,
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
    pub miracle_levels: [[u8; 8]; 4],
    pub god_item_texts: Vec<[Vec<u8>; 2]>,
    pub god_item_names: Vec<Vec<u8>>,
    pub god_name_text: Vec<u8>,
    pub god_bought_texts: [Vec<u8>; 2],
    pub god_short_texts: [Vec<u8>; 2],
    pub god_chatter_texts: Vec<[Vec<u8>; 2]>,
    pub god_intro_texts: Vec<[Vec<u8>; 2]>,
    pub menu_texts: Vec<Option<Texture>>,
    pub map_reopen_times: BTreeMap<i32, f64>,
    pub item_drop_queue: Vec<Vec<i32>>,
    pub drop_icons: BTreeMap<i32, Option<Rc<Imgcut>>>,
    pub web_popup_entries: Vec<WebPopupEntry>,
    pub web_popup_shown: Vec<[i32; 2]>,
    pub ad_button_id: i32,
    pub ad_button_cleared: u8,
    pub lineup_stages: BTreeMap<i16, Vec<i32>>,
    pub lineup_records: BTreeMap<i16, LineupRecord>,
    pub labyrinth_units: Vec<i32>,
    pub labyrinth_units_used: BTreeMap<i32, bool>,
    pub labyrinth_floors: BTreeMap<i32, LabyrinthFloor>,
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
    pub stage_unlock_cache: BTreeMap<i32, [i16; 4]>,
    pub stage_record_neg20: Vec<i16>,
    pub stage_record_neg19: Vec<i16>,
    pub stage_record_neg18: Vec<i16>,
    pub stage_record_neg17: Vec<i16>,
    pub stage_record_neg16: Vec<i16>,
    pub stage_record_neg11: Vec<i16>,
    pub stage_record_neg10: Vec<i32>,
    pub stage_record_neg9: Vec<i32>,
    pub stage_record_neg4: Vec<i32>,
    pub stage_table_neg26: StageNameTable,
    pub stage_table_neg24: StageNameTable,
    pub stage_table_neg23: StageNameTable,
    pub stage_table_neg22: StageNameTable,
    pub map_flag_neg4: Vec<u8>,
    pub map_flag_neg9: Vec<i32>,
    pub map_flag_neg10: Vec<i32>,
    pub map_flag_neg11: Vec<u8>,
    pub map_flag_neg16: Vec<u8>,
    pub map_flag_neg17: Vec<u8>,
    pub map_flag_neg18: Vec<u8>,
    pub map_flag_neg20: Vec<u8>,
    pub map_count_neg4: Vec<i32>,
    pub map_count_neg10: Vec<i32>,
    pub map_count_neg11: Vec<i32>,
    pub map_count_neg16: Vec<i32>,
    pub map_count_neg17: Vec<i32>,
    pub map_count_neg18: Vec<i32>,
    pub map_count_neg20: Vec<i32>,
    pub map_count_neg21: Vec<i32>,
    pub map_stamp_neg4: Vec<i64>,
    pub map_stamp_neg10: Vec<i64>,
    pub map_stamp_neg11: Vec<i64>,
    pub map_stamp_neg16: Vec<i64>,
    pub map_stamp_neg17: Vec<i64>,
    pub map_stamp_neg18: Vec<i64>,
    pub map_stamp_neg20: Vec<i64>,
    pub map_stamp_neg21: Vec<i64>,
    pub map_stamp_3: Vec<i64>,
    pub map_stamp_4: Vec<i64>,
    pub map_opened_neg4: Vec<bool>,
    pub map_opened_neg10: Vec<bool>,
    pub map_opened_neg11: Vec<bool>,
    pub map_opened_neg16: Vec<bool>,
    pub map_opened_neg17: Vec<bool>,
    pub map_opened_neg18: Vec<bool>,
    pub map_opened_neg20: Vec<bool>,
    pub map_opened_neg21: Vec<bool>,
    pub map_opened_3: Vec<bool>,
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
    pub const ITEM_COUNTS_KIND_D: usize = 0x1b0;
    pub const ITEM_CF_COUNT: usize = 0x478;
    pub const ITEM_COUNTS_KIND_B: usize = 0x10ac;
    pub const DECK_BUTTON_HELD: usize = 0x1404;
    pub const DECK_TWO_LINES: usize = 0x1405;
    pub const ITEM_COUNTS_KIND_9: usize = 0x140a;
    pub const ITEM_COUNTS_KIND_9_STRIDE: usize = 0x18;
    pub const ITEM_7B_COUNT: usize = 0x1ced;
    pub const ITEM_69_COUNT: usize = 0x1eea;
    pub const ITEM_1D_COUNT: usize = 0x2658;
    pub const ITEM_91_COUNT: usize = 0x2660;
    pub const ITEM_9D_COUNT: usize = 0x2668;
    pub const ITEM_D4_COUNT: usize = 0x2670;
    pub const BATTLE_ZOOM_Y: usize = 0x34a4;
    pub const ITEM_COUNTS_KIND_8: usize = 0x34a8;
    pub const ITEM_COUNTS_KIND_A: usize = 0x3660;
    pub const ITEM_COUNTS_KIND_C: usize = 0x3670;
    pub const ITEM_16_COUNT: usize = 0xc248;
    pub const ITEM_6_COUNT: usize = 0xc3b8;
    pub const ITEM_7_COUNT: usize = 0xc3c0;
    pub const ITEM_COUNTS_KIND_3: usize = 0x4a5b0;
    pub const ITEM_14_COUNT: usize = 0x3346e4;
    pub const ITEM_15_COUNT: usize = 0x3346ec;
    pub const ITEM_COUNTS_KIND_1: usize = 0x3346f4;
    pub const DEPLOY_NOTICE_TIMER: usize = 0x2971f0;
    pub const DEPLOY_NOTICE_KIND: usize = 0x2971f4;
    pub const BABY_BOOM_ACTIVE: usize = 0x333134;
    pub const MEDAL_MONEY_0: usize = 0x19f8;
    pub const MEDAL_MONEY_1: usize = 0x19fc;
    pub const MEDAL_MONEY_4: usize = 0x1a00;
    pub const DEPLOY_LIMIT_RARITY_COUNTS: usize = 0x3432a0;
    pub const DEPLOY_LIMIT_TOTAL: usize = 0x3432b8;
    pub const GATYA_ITEM_ROWS: usize = 0x392460;
    pub const ITEM_DEFINITIONS: usize = 0x392484;
    pub const ITEM_REDIRECT_SCALES: usize = 0x392478;
    pub const MISSION_CANNON_FIRED: usize = 0x3c3330;
    pub const ITEM_DEFINITION_STRIDE: usize = 0x40;
    pub const ITEM_5C_COUNT: usize = 0x430168;
    pub const ITEM_5C_CAP: usize = 0x431288;
    pub const ITEM_COUNTS_KIND_5: usize = 0x448a2c;
    pub const ITEM_COUNTS_KIND_6: usize = 0x448a5c;
    pub const ITEM_COUNTS_KIND_7: usize = 0x448a74;
    pub const UNITS_OWNED: usize = moved(0x46dd8, TAIL_UNITS_OWNED);
    pub const UNIT_LEVELS: usize = moved(0x47ba4, TAIL_UNIT_LEVELS);
    pub const TECH_LEVELS: usize = 0x4a4fc;
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
    pub const INPUT_BLOCKED: usize = 0x32e01c;
    pub const OPTION_MENU_IS_OPEN: usize = 0x32e044;
    pub const SWIPE_DY: usize = 0x32e060;
    pub const SWIPE_ANGLE: usize = 0x32e06c;
    pub const SWIPE_VELOCITY: usize = 0x32e070;
    pub const DECK_ROW_SWAP_DIRECTION: usize = 0x32e07c;
    pub const CAMERA_KICK: usize = 0x32e080;
    pub const SNIPER_TARGET: usize = 0x32e0a0;
    pub const SNIPER_BOB_ANGLE: usize = 0x32e0a4;
    pub const SNIPER_AIM_ANGLE: usize = 0x32e0a8;
    pub const SNIPER_AIM_GOAL: usize = 0x32e0ac;
    pub const PENDING_STRIKE_TRIGGER_X: usize = 0x32e0b0;
    pub const PENDING_STRIKE_Y: usize = 0x32e178;
    pub const PENDING_STRIKE_TARGET: usize = 0x32e240;
    pub const PENDING_STRIKE_ACTIVE: usize = 0x32e308;
    pub const PENDING_STRIKE_SPEED: usize = 0x32e33c;
    pub const PENDING_STRIKE_ANGLE: usize = 0x32e404;
    pub const SNIPER_CHARGE: usize = 0x32e4cc;
    pub const SNIPER_FIRING: usize = 0x32e4d0;
    pub const SNIPER_FIRE_FRAME: usize = 0x32e4d4;
    pub const SNIPER_RECOIL: usize = 0x32e4d8;
    pub const SNIPER_CASINGS: usize = 0x32e4dc;
    pub const SNIPER_CASINGS_LIVE: usize = 0x32e9dc;
    pub const DECK_ROW_SHOWN: usize = 0x32e9e0;
    pub const DECK_ROW_SWAP_OFFSETS: usize = 0x32e9e4;
    pub const DECK_ROW_SWAP_FRAME: usize = 0x32e9f4;
    pub const DECK_ROW_SWAP_TARGET: usize = 0x32e9f8;
    pub const DECK_ROW_SWAPPING: usize = 0x32e9fc;
    pub const DECK_ROW_SWAP_TAIL: usize = 0x32ea02;
    pub const DECK_SWIPE_LATCHED: usize = 0x32e9fe;
    pub const PINCH_ZOOMED: usize = 0x32e9ff;
    pub const CAMERA_DRAGGING: usize = 0x32ea00;
    pub const PENDING_STRIKE_SPARKS: usize = 0x32e52c;
    pub const PENDING_STRIKE_SPARKS_STRIDE: usize = 0x18;
    pub const DRAW_TEMP_0: usize = 0x32f7dc;
    pub const DRAW_TEMP_1: usize = 0x32f7e0;
    pub const DRAW_TEMP_2: usize = 0x32f7e4;
    pub const DRAW_TEMP_3: usize = 0x32f7e8;
    pub const DRAW_TEMP_4: usize = 0x32f7ec;
    pub const DRAW_TEMP_5: usize = 0x32f7f0;
    pub const DRAW_TEMP_6: usize = 0x32f7f4;
    pub const DRAW_TEMP_7: usize = 0x32f7f8;
    pub const ANCHOR_OUT: usize = 0x3400;
    pub const DRAW_LIST: usize = 0x4a6f0;
    pub const DRAW_SWAP: usize = 0x4b050;
    pub const DECK_BAR_SLIDE: usize = 0x32f804;
    pub const LETTERBOX_SHIFT: usize = 0x32f9a8;
    pub const OUTRO_EXIT_DIRECT: usize = 0x1ee8;
    pub const OUTRO_EXIT_EVENT: usize = 0x1ee9;
    pub const OUTRO_MAP_LOCKED: usize = 0x83e50;
    pub const OUTRO_VIDEO_BUTTON: usize = 0x880;
    pub const OUTRO_VIDEO_WATCHED: usize = 0x881;
    pub const LAST_VIDEO_TIME: usize = 0x888;
    pub const LEADERSHIP_REFUND: usize = 0x420;
    pub const LOSE_BANNER_Y: usize = 0x32e068;
    pub const LOSE_CHOICE: usize = 0x32f9b0;
    pub const LOSE_NO_PRESS: usize = 0x32fd70;
    pub const LOSE_SHOP_PRESS: usize = 0x32fda4;
    pub const LOSE_SHOP_X: usize = 0x32e054;
    pub const LOSE_SHOP_RECT: usize = 0x32fd1c;
    pub const LOSE_SHOP_HELD: usize = 0x331e7a;
    pub const LOSE_RECORDED: usize = 0xc3cc;
    pub const LOSE_TIP: usize = 0x32f940;
    pub const EX_OFFERED: usize = 0x40a510;
    pub const CAT_FOOD_SHOP_ENABLED: usize = 0xc3ec;
    pub const CAT_FOOD_SHOP_OPEN: usize = 0x331eaa;
    pub const CAT_FOOD_SHOP_MODE: usize = 0x331ec4;
    pub const SHOP_TUTORIAL_SEEN: usize = 0x4a600;
    pub const PENDING_SCENE: usize = 0x3388;
    pub const PENDING_SCENE_ARMED: usize = 0x338c;
    pub const SCENE_CHANGE_REQUESTED: usize = 0x3390;
    pub const REVIVE_REQUESTED: usize = 0x32f93c;
    pub const FIRST_STAGE_WON: usize = 0xc3d4;
    pub const NEXT_STAGE_UNLOCKED: usize = 0x83e48;
    pub const WIN_TREASURE: usize = 0x83e44;
    pub const RANK_POPUP_SHOWN: usize = 0x39703c;
    pub const RANK_REWARD_BASE: usize = 0x3c3338;
    pub const EX_ROLLED: usize = 0x40a511;
    pub const EX_ACCEPTED: usize = 0x40a512;
    pub const EX_MAP: usize = 0x40a514;
    pub const EX_STAGE: usize = 0x40a51c;
    pub const OUTRO_FRAME: usize = 0x83e3c;
    pub const OUTRO_TICKS: usize = 0x3c336c;
    pub const BATTLE_RESUMED: usize = 0x38fb88;
    pub const BATTLE_CONTINUED: usize = 0x3343f0;
    pub const POINT_LIMIT_PENDING: usize = 0x248;
    pub const OUTRO_OK_PRESS: usize = 0x32fd6c;
    pub const OUTRO_OK_RECT: usize = 0x32fc7c;
    pub const OUTRO_OK_SLIDE: usize = 0x32f808;
    pub const OUTRO_BUTTON_LOCK: usize = 0x40a2cc;
    pub const REWARD_POP_HOLD: usize = 0x32ff70;
    pub const LABYRINTH_RESULT_READY: usize = 0x1090;
    pub const LABYRINTH_RANK: usize = 0x106c;
    pub const LABYRINTH: usize = 0xaf8;
    pub const LABYRINTH_MAP_ID: usize = 0x1048;
    pub const LABYRINTH_FLOOR_REACHED: usize = 0x1064;
    pub const LABYRINTH_FLOOR_BEST: usize = 0x1068;
    pub const LABYRINTH_STAGE_IDS: usize = 0xe58;
    pub const LABYRINTH_SLOTS: usize = 0x64;
    pub const DECK_BACK_ROW_ENABLED: usize = 0x332e12;
    pub const DRAG_LATCHED: usize = 0x334364;
    pub const CPU_ENABLED: usize = 0x32ff98;
    pub const CPU_PENDING_ACTION: usize = 0x32ff9c;
    pub const CPU_PICK: usize = 0x32ffa4;
    pub const CPU_USABLE: usize = 0x32ffac;
    pub const CPU_FACTION_STRIDE: usize = 0x28;
    pub const CPU_SAVING_FOR: usize = 0x32fffc;
    pub const CPU_CANNON_STATE: usize = 0x330004;
    pub const CPU_CANNON_WAIT: usize = 0x33000c;
    pub const CPU_CANDIDATES: usize = 0x33001c;
    pub const SCENE_0X64_PAGE: usize = 0x32ff0c;
    pub const UI_TAP_LOCKOUT: usize = 0x331edc;
    pub const CAT_GOD_MENU_IS_OPEN: usize = 0x332f14;
    pub const CANNON_BLAST_ACTIVE: usize = 0x33312c;
    pub const BASE_GUARD_NOTICE: usize = 0x898;
    pub const KILLS_SINCE_SPAWN_TICK: usize = 0x1120;
    pub const LABYRINTH_ACTIVE: usize = 0x101c;
    pub const SCORE_MODE_FLAG: usize = 0x32e9;
    pub const SCORE_TOTAL: usize = 0x32ec;
    pub const SCORE_ELAPSED: usize = 0x32f0;
    pub const SCORE_CHANGED: usize = 0x3310;
    pub const SCORE_ANIM_TICK: usize = 0x3314;
    pub const SCORE_SHOWN: usize = 0x3318;
    pub const SCORE_FROM: usize = 0x331c;
    pub const SCORE_ZOOM: usize = 0x3320;
    pub const LINEUP_CANNON_TYPE: usize = 0x4d8;
    pub const LINEUP_CANNON_LEVEL: usize = 0x4dc;
    pub const EX_REDIRECT_A_BLOCKED: usize = 0x14b8;
    pub const BUILT_DECK_EX_STAGE_KEY: usize = 0x32b0;
    pub const USE_BUILT_DECK: usize = 0x32b4;
    pub const STORY_MAP_COUNTS: usize = 0x3394;
    pub const UNIT_INFO_OVERLAY_OPEN: usize = 0x938;
    pub const ITF_PROGRESS: usize = 0xca4c;
    pub const COTC_PROGRESS: usize = 0xca58;
    pub const STAGES_CLEARED_CHAPTERS_KEY: usize = 0xca64;
    pub const PINCH: usize = 0x3408;
    pub const CAMERA_ZOOM: usize = 0x343c;
    pub const TOUCH_X: usize = 0x3458;
    pub const TOUCH_START_X: usize = 0x3460;
    pub const TOUCH_PREV_X: usize = 0x3464;
    pub const TOUCH_Y: usize = 0x3468;
    pub const TOUCH_START_Y: usize = 0x3470;
    pub const TOUCH_PREV_Y: usize = 0x3474;
    pub const TOUCH_PENDING_X: usize = 0x345c;
    pub const TOUCH_PENDING_Y: usize = 0x346c;
    pub const TOUCH_PENDING_BEGAN: usize = 0x3479;
    pub const TOUCH_PENDING_RELEASED: usize = 0x347c;
    pub const TOUCH_DOWN_LATCH: usize = 0x347d;
    pub const TOUCH_BEGAN: usize = 0x3478;
    pub const TOUCH_IS_DOWN: usize = 0x347a;
    pub const TOUCH_RELEASED: usize = 0x347b;
    pub const BACK_PRESSED: usize = 0x347e;
    pub const SCENE_ID: usize = 0x3480;
    pub const DECK_PRESETS: usize = 0xc400;
    pub const DECK_PRESET_STRIDE: usize = 0x2c;
    pub const DECK_PRESET_KEY: usize = 0xc428;
    pub const FACTION_1_DECK: usize = 0xc42c;
    pub const BATTLE_DECK: usize = 0xc79c;
    pub const STAGES_CLEARED_CHAPTERS: usize = 0xca3c;
    pub const STAGE_RECORD_CHAPTERS: usize = 0xca68;
    pub const SEEN_ENEMIES: usize = moved(0xda58, TAIL_SEEN_ENEMIES);
    pub const FACTION_1_UNIT_FORMS: usize = Self::UNIT_FORMS - 8;
    pub const UNIT_FORMS: usize = moved(0x49734, TAIL_UNIT_FORMS);
    pub const BATTLE_FRAME_COUNTER: usize = 0x83e00;
    pub const CAMERA_X: usize = 0x83e10;
    pub const AUTO_CAMERA_MODE: usize = 0x83e2c;
    pub const BATTLE_STATUS: usize = 0x83e34;
    pub const WORKER_UPGRADE_VFX: usize = 0x83e60;
    pub const CAMERA_MIN_ZOOM: usize = 0x83e6c;
    pub const CASTLE_ID: usize = 0x83e4c;
    pub const STAGE_CASTLE_ID: usize = 0x83e84;
    pub const TREASURE_PROGRESS: usize = 0x83e90;
    pub const SPAWN_COUNTDOWN: usize = 0x84048;
    pub const CAT_DEBRIS: usize = 0x9d088;
    pub const DEBRIS_STRIDE: usize = 0x10;
    pub const ENEMY_DEBRIS: usize = 0x9d408;
    pub const CANNON_SHOTS: usize = 0x9d788;
    pub const CANNON_SHOTS_FACTION_STRIDE: usize = 0xb4;
    pub const CANNON_SHOT_STRIDE: usize = 0xc;
    pub const STAGE_LENGTH: usize = 0x9ee48;
    pub const STAGE_SPAWN_MIN: usize = 0x9ee50;
    pub const STAGE_SPAWN_MAX: usize = 0x9ee54;
    pub const STAGE_BACKGROUND_ID: usize = 0x9ee58;
    pub const STAGE_MAX_ENEMIES: usize = 0x9ee5c;
    pub const CASTLE_ENEMY_ROW: usize = 0x9ee60;
    pub const STAGE_SCORE_TIME_LIMIT: usize = 0x9ee64;
    pub const STAGE_BOSS_GUARD: usize = 0x9ee68;
    pub const SCENE_0X63_STATE: usize = 0x32d64c;
    pub const INSETS_IGNORED: usize = 0x2100;
    pub const STAGE_INDEX: usize = 0x32d668;
    pub const CHAPTER_MODE: usize = 0x32f934;
    pub const FACTION_1_BUTTON_ROWS: usize = 0x32f950;
    pub const POWERUPS: usize = 0x32f8e8;
    pub const POWERUP_AVAILABLE: usize = 0x32f924;
    pub const SPEED_UP_LATCH: usize = 0x32f927;
    pub const BUTTON_UNIT_FORMS: usize = 0x32f978;
    pub const BATTLE_IS_OUTBREAK: usize = 0x334048;
    pub const OUTBREAKS_ENABLED: usize = 0x334049;
    pub const BATTLE_IS_INVASION: usize = 0x334056;
    pub const BATTLE_IS_Z_INVASION: usize = 0x334057;
    pub const INVASION_STAGE: usize = 0x334058;
    pub const MAP_NEG15_CLEARED: usize = 0x334059;
    pub const MAP_NEG25_CLEARED: usize = 0x33405a;
    pub const WAVE_RECORDS: usize = 0x33b81c;
    pub const WAVE_RECORD_STRIDE: usize = 0x30;
    pub const WAVE_SPRITES: usize = 0x33dd9c;
    pub const MAP_INDEX: usize = 0x340368;
    pub const STAGES_CLEARED_STORY: usize = 0x345ad0;
    pub const STAGES_CLEARED_NEG6: usize = 0x3481f8;
    pub const STAGE_RECORD_STORY: usize = 0x3482c8;
    pub const STAGE_RECORD_NEG6: usize = 0x382c60;
    pub const STAMP_DATA: usize = 0x333138;
    pub const SAVED_MAP_TYPE: usize = 0x38b164;
    pub const CHAPTER_COST_TIER: usize = 0x38fad8;
    pub const PROC_ROLLS: usize = 0x38fb8c;
    pub const WAVE_HITS: usize = 0x38fbc0;
    pub const SELECTED_DECK_PRESET: usize = 0x39781c;
    pub const PRESET_STYLE_PARTS: usize = 0x42f969;
    pub const PRESET_FOUNDATION_PARTS: usize = 0x42f96a;
    pub const CROWN_LEVEL: usize = 0x39797c;
    pub const EX_MAP_INDEX: usize = 0x40a518;
    pub const EX_STAGE_INDEX: usize = 0x40a520;
    pub const UNIT_LEVEL_CURVE: usize = moved(0x44fcc4, TAIL_UNIT_LEVEL_CURVE);
    pub const UNIT_EXP_CURVE: usize = moved(0x461064, TAIL_UNIT_EXP_CURVE);
    pub const STAGE_NO_CONTINUES: usize = 0x473494;
    pub const STAGE_EX_CHANCE: usize = 0x473498;
    pub const STAGE_EX_MAP: usize = 0x47349c;
    pub const STAGE_EX_STAGE_MIN: usize = 0x4734a0;
    pub const STAGE_EX_STAGE_MAX: usize = 0x4734a4;
    pub const STAGE_RECORD_NEG8: usize = 0x4734a8;
    pub const ITEM_DEFINITION_IDS: usize = 0x392470;
    pub const DECK_HOLD_FRAMES: usize = 0x13f8;
    pub const DECK_HOLD_SLOT: usize = 0x13fc;
    pub const DECK_HOLD_RELEASE: usize = 0x1400;
    pub const VIBRATION_ENABLED: usize = 0x249;
    pub const MAP_STAGE_ROWS: usize = 0x38b168;
    pub const MAP_STAGE_ROW_STRIDE: usize = 0xbc;
    pub const STAGE_ROW: usize = 0x83dfc;
    pub const STAGE_MUSIC_ROW: usize = 0x3c3988;
    pub const BGM_SWITCH_STATE: usize = 0x296edc;
    pub const BGM_SWITCH_FRAME: usize = 0x296ee0;
    pub const BGM_SWITCH_FRAMES: usize = 0x296ee4;
    pub const BGM_SWITCH_NEXT: usize = 0x296ee8;
    pub const BGM_BOSS_PHASE: usize = 0x296eec;
    pub const BGM_PENDING: usize = 0x474438;
    pub const BGM_DELAY_FRAME: usize = 0x47443c;
    pub const BGM_DELAY: usize = 0x474440;
    pub const ITEMS_SELECTED: usize = 0x32f910;
    pub const ITEMS_SELECTED_LABYRINTH: usize = 0x23a4;
    pub const ITEMS_SELECTED_SCORE_MODE: usize = 0x2397;
    pub const DEMON_BANNER_FRAME: usize = 0x12e0;
    pub const COMBO_BANNER_STEP_FRAMES: usize = 0x449300;
    pub const COMBO_BANNER_LIFETIME: usize = 0x449304;
    pub const COMBO_BANNER_SPEED: usize = 0x449308;
    pub const COMBO_BANNER_TEXT_STEP: usize = 0x44930c;
    pub const COMBO_BANNER_STATE: usize = 0x449310;
    pub const COMBO_BANNER_PHASE: usize = 0x449314;
    pub const COMBO_BANNER_X: usize = 0x449318;
    pub const COMBO_BANNER_SUB: usize = 0x44931c;
    pub const COMBO_BANNER_TICKS: usize = 0x449320;
    pub const COMBO_SKIP_RECT: usize = 0x32fc5c;
    pub const COMBO_BANNER_UNITS: usize = 0x449324;
    pub const CRIT_VFX: usize = 0x9d8f0;
    pub const CRIT_VFX_STRIDE: usize = 0x10;
    pub const ZKILL_VFX: usize = 0x9e570;
    pub const ZKILL_VFX_STRIDE: usize = 0x10;
    pub const BARRIER_VFX: usize = 0x9e750;
    pub const BARRIER_VFX_STRIDE: usize = 0x1c;
    pub const SHIELD_VFX: usize = 0x9ea98;
    pub const SHIELD_VFX_STRIDE: usize = 0x1c;
    pub const BASE_GUARD_NOTICE_FRAME: usize = 0x89c;
    pub const TUTORIAL_POPUP_OPEN: usize = 0x332ecc;
    pub const OPTION_WINDOW: usize = 0x472408;
    pub const OPTION_WINDOW_PAGE: usize = 0x472420;
    pub const RETURN_CONFIRM_OPEN: usize = 0x32e045;
    pub const UNIT_INFO_SAVED_TWO_LINES: usize = 0x939;
    pub const UNIT_INFO_SLOT: usize = 0x93c;
    pub const BATTLE_ENTRY_RESET: usize = 0x33462c;
    pub const CURTAIN_ACTIVE: usize = 0x32e038;
    pub const CURTAIN_STYLE: usize = 0x32e040;
    pub const BATTLE_TICKS: usize = 0x32ff5c;
    pub const BATTLE_INTRO_FRAME: usize = 0x334218;
    pub const CAMERA_MATRIX: usize = 0x3440;
    pub const CAMERA_OFFSET: usize = 0x3450;
    pub const BASE_KILL_BLOCKED: usize = 0x39a228;
    pub const AUTO_CAMERA_ARRIVED: usize = 0x32f9a0;
    pub const SETUP_FRAMES: usize = 0x32ff48;
    pub const SETUP_BLOCK: usize = 0x32ff44;
    pub const REWARD_POP_COUNTER: usize = 0x32f930;
    pub const SPEED: usize = 0x32f7d8;
    pub const CAT_GOD_HEAL_PENDING: usize = 0x333128;
    pub const BABY_BOOM_FRAMES: usize = 0x32f92c;
    pub const DEPLOY_FULL_FLASH: usize = 0x1d08;
    pub const HIT_COUNT: usize = 0x292ad8;
    pub const HIT_LIST: usize = 0x292938;
    pub const HIT_SWAP: usize = 0x292ad0;
    pub const OUTRO_PHASE: usize = 0x83e38;
    pub const LOSE_TIP_SHOWN: usize = 0x83e5c;
    pub const LOST_MAP_TYPE: usize = 0x32f944;
    pub const LOST_MAP_INDEX: usize = 0x32f948;
    pub const LOST_STAGE: usize = 0x32f94c;
    pub const DEFEAT_COUNTER: usize = 0x2f44;
    pub const EVENT_UNIT_OWNED: usize = 0x3969f4;
    pub const NEW_BEST_SCORE: usize = 0x83e58;
    pub const PLAY_FRAMES: usize = 0x83e74;
    pub const OPTION_WINDOW_KIND: usize = 0x47240c;
    pub const BG_IMAGE_ID: usize = 0x333264;
    pub const BG_MODEL_ID: usize = 0x33325c;
    pub const TECH_MAX_LEVELS: usize = 0x3923a0;
    pub const UNIT_FORM_COUNTS: usize = moved(0x403470, TAIL_UNIT_FORM_COUNTS);
    pub const COMBO_PAGE_LIST: usize = 0x333cf0;
    pub const COMBO_UNLOCK_NOTICE: usize = 0x4493ac;
    pub const COMBO_TAB: usize = 0x4492a4;
    pub const LEGEND_STAGE_ENERGY: usize = 0x473304;
    pub const LEGEND_STAGE_INFO: usize = 0x472b34;
    pub const BG_SETUP: usize = 0x33324c;
    pub const BG_TINT_XS: usize = 0x3300a8;
    pub const BG_TINT_YS: usize = 0x3300b8;
    pub const BG_TINT_COLORS: usize = 0x3300c8;
    pub const OUTRO_RECTS: usize = 0x32fd0c;
    pub const OPTION_RECTS: usize = 0x32fcdc;
    pub const DECK_SWAP_BLOCK: usize = 0x330088;
    pub const UI_STATE_TAIL: usize = 0x333248;
    pub const HUD_RECTS: usize = 0x32fe08;
    pub const TUTORIAL_PAGE: usize = 0x332ee0;
    pub const DRAW_SCENE_ID: usize = 0x3484;
    pub const INSET_STRIPS_DUE: usize = 0x2101;
    pub const TOUCH_ID: usize = 0x32f9a4;
    pub const MAIN_DRAW_HIDDEN: usize = 0x33fc;
    pub const MAP_STAMINA_HUD_STATE: usize = 0x1d16;
    pub const TUTORIAL_POPUP_FRAME: usize = 0x332ed4;
    pub const TUTORIAL_BOB_PHASE: usize = 0x332ed8;
    pub const TUTORIAL_PRESS_TICKS: usize = 0x332edc;
    pub const TUTORIAL_BUTTON_X: usize = 0x332ee4;
    pub const TUTORIAL_BUTTON_Y: usize = 0x332ee8;
    pub const TUTORIAL_BUTTON_W: usize = 0x332eec;
    pub const TUTORIAL_BUTTON_H: usize = 0x332ef0;
    pub const TUTORIAL_BUTTON_HELD: usize = 0x331e9f;
    pub const TUTORIAL_FORMATION_SEEN: usize = 0x4a5f8;
    pub const TUTORIAL_ROW12_SEEN: usize = 0x4a610;
    pub const TUTORIAL_ROW10_SEEN: usize = 0x4a618;
    pub const TUTORIAL_SCENE_JUMP_SEEN: usize = 0x4a628;
    pub const TUTORIAL_MISSION_SEEN: usize = 0x4a638;
    pub const TUTORIAL_EXIT_STATE: usize = 0x397054;
    pub const CAT_GOD_INTRO_BUTTON_PRESS: usize = 0x33310c;
    pub const SHOP_UPDATE_CONSUMED: usize = 0x334648;
    pub const RESTRICTION_WARNING_TEXTS: usize = 0xb8f0;
    pub const STAGE_RECORD_CHAPTERS_KEY: usize = 0xcb34;
    pub const STAGE_CLEAR_FLAG: usize = 0xc3d0;
    pub const POWERUP_USED: usize = 0x3c3331;
    pub const STAGE_BASE_HP: usize = 0x9ee4c;
    pub const BATTLE_CLOCK: usize = 0x9ee40;
    pub const CAT_GOD_GLOW_TIMER: usize = 0x332f0c;
    pub const LINEUP_BASE_LEVEL: usize = 0x4e0;
    pub const POWERUP_FREE: usize = 0x32f91a;
    pub const POWERUP_GRANTS: usize = 0x1408;
    pub const RESUMED_MONEY: usize = 0x83e24;
    pub const BATTLE_DECK_KEY: usize = 0xc7c4;
    pub const EX_REDIRECT_ENABLED: usize = 0x3c40c1;
    pub const SCENE_4_PAGE: usize = 0x3324;
    pub const ALL_MAPS_OPEN: usize = 0x3976c0;
    pub const MAP_COORDS: usize = 0x296ef0;
    pub const POLYGON_YS: usize = 0x3300e4;
    pub const POLYGON_XS: usize = 0x3300d8;
    pub const LETTERBOX_PAD: usize = 0x32f9ac;
    pub const MIRACLE_PRICES: usize = 0x474450;
    pub const CAT_GOD_TICKS: usize = 0x332f2c;
    pub const CAT_GOD_DROP_SPEED: usize = 0x332fb4;
    pub const CAT_GOD_FLASH_X: usize = 0x332f98;
    pub const CAT_GOD_RETURN_STEP: usize = 0x332f88;
    pub const CAT_GOD_RETURN_TICKS: usize = 0x332f84;
    pub const CAT_GOD_SAVED_CAMERA_X: usize = 0x332f80;
    pub const CAT_GOD_SAVED_ZOOM: usize = 0x332f7c;
    pub const CAT_GOD_ZOOM_DELTA: usize = 0x332f74;
    pub const CAT_GOD_ZOOM_TICKS: usize = 0x332f78;
    pub const CAT_GOD_ZOOM_DONE: usize = 0x332f8c;
    pub const CAT_GOD_FADE: usize = 0x332f24;
    pub const CAT_GOD_PUSHING: usize = 0x333130;
    pub const CAT_GOD_SHAKE_Y: usize = 0x33300c;
    pub const CAT_GOD_SHAKE_X: usize = 0x332fbc;
    pub const CAT_GOD_FRAMES: usize = 0x332f40;
    pub const CAT_GOD_ANIM_FRAME: usize = 0x332f3c;
    pub const CAT_GOD_HOVER: usize = 0x331e8f;
    pub const CAT_GOD_BACK_RECT: usize = 0x3330cc;
    pub const CAT_GOD_CONFIRM_RECT: usize = 0x3330bc;
    pub const CAT_GOD_CLOSE_RECT: usize = 0x3330ac;
    pub const CAT_GOD_MIRACLE_RECTS: usize = 0x33306c;
    pub const CAT_GOD_CONFIRM_OPEN: usize = 0x332f15;
    pub const CAT_GOD_SELECTED: usize = 0x333124;
    pub const CAT_GOD_PRESSES: usize = 0x333100;
    pub const CAT_GOD_OPEN_TICKS: usize = 0x332f94;
    pub const CAT_GOD_IDLE_FRAMES: usize = 0x332f90;
    pub const CAT_GOD_BOB: usize = 0x332fa8;
    pub const CAT_GOD_VELOCITY: usize = 0x332fb8;
    pub const CAT_GOD_OFFSET: usize = 0x332fa4;
    pub const CAT_GOD_SETTLE_FRAME: usize = 0x332f38;
    pub const CAT_GOD_CHATTER_TIMER: usize = 0x332f30;
    pub const CAT_GOD_SPIN_SPEED: usize = 0x332f1c;
    pub const CAT_GOD_STATE: usize = 0x332f28;
    pub const DECK_COOLDOWN_VFX: usize = 0x27a8;
    pub const BGM_PLAYER: usize = 0x474428;
    pub const LEADERSHIP_TOTAL: usize = 0x41c;
    pub const OUTRO_EXIT_TARGET: usize = 0xc3fc;
    pub const LOSE_ENTRY_CHAPTER: usize = 0xc3f0;
    pub const MENU_BUILD_MODE: usize = 0x33622c;
    pub const MENU_CURSOR: usize = 0x4494b8;
    pub const MENU_PICKED: usize = 0x4494c8;
    pub const MENU_TEXTURE_PAGE: usize = 0x4494a0;
    pub const MENU_ANCHOR: usize = 0x4494d0;
    pub const MAP_RETURN_FLAG: usize = 0x340370;
    pub const HUD_STATE: usize = 0x32fed8;
    pub const SWIPE_STATE: usize = 0x32e078;
    pub const SCROLL_STATE: usize = 0x32e04c;
    pub const FADE_MENU_STATE: usize = 0x32e048;
    pub const SCENE_0X64_PAGE_NEXT: usize = 0x32ff14;
    pub const FADE_STARTED: usize = 0x2258;
    pub const FADE_FRAME: usize = 0x32e03c;
    pub const SCENE_LATCHES: usize = 0x3341b0;
    pub const ITEM_HOLD_SCORE_MODE: usize = 0x2390;
    pub const ITEM_HOLD_LABYRINTH: usize = 0x239d;
    pub const ITEM_HOLD_NORMAL: usize = 0x3340c4;
    pub const POWERUP_CLEARED: usize = 0x32f928;
    pub const LABYRINTH_RANKING: usize = 0x1098;
    pub const LEADERSHIP_NOTICE: usize = 0x3289;
    pub const STAMINA_HALVED: usize = 0x33436b;
    pub const DROP_MAP_STAGES: usize = 0x389910;
    pub const SPECIAL_BEST_SCORES: usize = 0x389f50;
    pub const FESTIVAL_STAGES: usize = 0x334188;
    pub const TREASURE_FESTIVAL_ENABLED: usize = 0x334365;
    pub const STAGE_SCORE: usize = 0x83e54;
    pub const CHAPTER_BEST_SCORES: usize = 0x3c3720;
    pub const RANKING_BEST_SCORES: usize = 0x3b7680;
    pub const RANKING_ID: usize = 0x3c3334;
    pub const SCORE_RANK_STATUSES: usize = 0x3c3340;
    pub const MAP_STAGE_SET: usize = 0x396954;
    pub const MAP_DATA_ID: usize = 0x396950;
    pub const TUTORIAL_STAGE_SIX: usize = 0x44fc20;
    pub const FIRST_WIN_PENDING: usize = 0x24a;
    pub const FIRST_WIN_GATE_B: usize = 0xc994;
    pub const FIRST_WIN_GATE_A: usize = 0x4a6b8;
    pub const EVENT_REWARD_ID: usize = 0x3c32c8;
    pub const REWARD_STATUS: usize = 0x3c333c;
    pub const DROP_FLAG: usize = 0x3343ec;
    pub const DROP_ROLL: usize = 0x3343e8;
    pub const DROP_RATE: usize = 0x3343e4;
    pub const UNIT_UNLOCKED_BY_CLEAR: usize = 0x3343d8;
    pub const UNIT_UNLOCK_NOTICE: usize = 0x4b05c;
    pub const OUTRO_NEW_CLEAR: usize = 0x3343dc;
    pub const OUTRO_NEW_UNLOCK: usize = 0x3343e0;
    pub const OUTRO_STAGE_CLEARED: usize = 0x83e78;
    pub const OUTRO_ENTRY_STAGE: usize = 0x32d65c;
    pub const OUTRO_CHAPTER_MODE: usize = 0x32f938;
    pub const AD_CONFIRM_DECLINED: usize = 0x900;
    pub const WIN_XP: usize = 0x83e40;
    pub const PRESET_CANNON_PARTS: usize = 0x42f968;
    pub const BATTLE_LINEUP: usize = 0xc770;
    pub const LABYRINTH_UNIT_COUNT: usize = 0xe38;
    pub const LABYRINTH_FLOOR_RESULT: usize = 0xe34;
    pub const EVENT_REWARDS_NEG2: usize = 0x3c3200;
    pub const EVENT_REWARDS: usize = 0x3b2b68;
    pub const CLEAR_LINEUP: usize = 0xc744;
    pub const TREASURE_LEVELS_STRIDE: usize = 0xc8;
    pub const TREASURE_LEVELS: usize = 0xd288;
    pub const REPLAY_MODE: usize = 0x397980;
    pub const FESTIVAL_COTC: usize = 0x334294;
    pub const FESTIVAL_ITF: usize = 0x334288;
    pub const FESTIVAL_EOC: usize = 0x33427c;
    pub const MAP_OPEN_NEG6: usize = 0x385798;
    pub const MAP_OPEN_STORY: usize = 0x383070;
    pub const REWARD_FORMS_OWNED: usize = moved(0x3b1d70, TAIL_REWARD_FORMS_OWNED);
    pub const REWARD_UNITS_OWNED: usize = moved(0x3b0fa8, TAIL_REWARD_UNITS_OWNED);
    pub const UNITS_UNLOCKED_FLAG: usize = 0x4a558;
    pub const REWARD_1002_OWNED: usize = 0x334644;
    pub const STAGE_UNLOCK_CHAPTERS: usize = 0xca14;
    pub const STAGE_UNLOCK_NEG3: usize = 0xca24;
    pub const STAGE_UNLOCK_NEG7: usize = 0xca30;
    pub const STAGE_UNLOCK_NEG6: usize = 0x3459e8;
    pub const STAGE_UNLOCK_STORY: usize = 0x3432c0;
    pub const WALLET_RED_GAUGE_FRAMES: usize = 0x1a8;
    pub const WALLET_ORB_DEPLOYS_SEEN: usize = 0x158;
    pub const DRAW_FRAMES: usize = 0x32f9bc;
    pub const BLINK_COUNTER: usize = 0x32f9b4;
    pub const BLINK_ON: usize = 0x32f9b8;
    pub const CAT_GOD_SPIN: usize = 0x332f18;
    pub const CAT_GOD_GLOW: usize = 0x332f20;
    pub const TUTORIAL_CLEARED: usize = 0xc3c8;
    pub const TUTORIAL_STEP: usize = 0x24c;
    pub const TUTORIAL_DECK_SEEN: usize = 0x4a5ec;
    pub const TUTORIAL_TWO_ROWS_SEEN: usize = 0x4a640;
    pub const GAUGE_TUTORIAL_STEP: usize = 0x4a65c;
    pub const TUTORIAL_TIMER: usize = 0x332ed0;
    pub const TUTORIAL_CAT_GOD_SEEN: usize = 0x4a5f0;
    pub const CAT_GOD_AVAILABLE: usize = 0xc3e4;
    pub const ENTRY_STAGE: usize = 0x32d658;
    pub const CAT_GOD_BUTTON_PRESS: usize = 0x3330fc;
    pub const CAT_GOD_INTRO_STEP: usize = 0x4a604;
    pub const PAUSE_PRESS: usize = 0x32fd78;
    pub const SPEED_UP_PRESS: usize = 0x32f8bc;
    pub const CPU_PRESS: usize = 0x32f8c8;
    pub const SNIPER_PRESS: usize = 0x32f8d0;
    pub const DECK_PRESS: usize = 0x32fdb8;
    pub const CAT_GOD_BUTTON_SINK: usize = 0x332fa0;
    pub const CAT_GOD_BUTTON_RECT: usize = 0x33305c;
    pub const TOUCH_CAPTURED: usize = 0x32e9fd;
    pub const ITEM_RECTS: usize = 0x32f80c;
    pub const TOOLTIP_ITEM: usize = 0x32e020;
    pub const TOOLTIP_PAGE: usize = 0x331ed8;
    pub const CANNON_RECT: usize = 0x32fc3c;
    pub const WORKER_RECT: usize = 0x32fc4c;
    pub const PAUSE_RECT: usize = 0x32fc6c;
    pub const CANNON_HELD: usize = 0x331e6c;
    pub const HELD_FLAGS_TAIL: usize = 0x331e7b;
    pub const WORKER_HELD: usize = 0x331e6d;
    pub const PAUSE_HELD: usize = 0x331e6f;
    pub const DECK_BUTTON_PRESSED: usize = 0x331e70;
    pub const BG_PARTICLES: usize = 0x292ae4;
    pub const BG_DRIFTERS: usize = 0x29337c;
    pub const BG_STARS: usize = 0x29355c;
    pub const BG_SPRITES: usize = 0x2939bc;

    pub fn new() -> Self {
        let mut raw = vec![0u8; SIZE].into_boxed_slice();

        for (field, value) in [
            (Self::COMBO_BANNER_STEP_FRAMES, 0x1ei32),
            (Self::COMBO_BANNER_LIFETIME, 0x78),
            (Self::COMBO_BANNER_SPEED, 0xc0),
            (Self::COMBO_BANNER_TEXT_STEP, 6),
        ] {
            raw[field..field + 4].copy_from_slice(&value.to_le_bytes());
        }

        for cell in 0..DROP_MAP_CELLS {
            let field = Self::DROP_MAP_STAGES + cell * 4;

            raw[field..field + 4].copy_from_slice(&(-1i32).to_le_bytes());
        }

        Self {
            raw,
            rng_state: 0,
            limits: Default::default(),
            treasure_gauge: Default::default(),
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
            cleared_session_keys: Vec::new(),
            stage_conditions: BTreeMap::new(),
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
            enemy_names: vec![Vec::new(); 2],
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
            buttons: ButtonBank {
                enabled: 1,
                ..ButtonBank::default()
            },
            unlock_popups: BTreeMap::new(),
            img001_sheet: None,
            outro_event_sheets: Default::default(),
            texture_cache: BTreeMap::new(),
            reward_queue: Vec::new(),
            item_possession: BTreeMap::new(),
            ex_groups: BTreeMap::new(),
            ex_lottery: Vec::new(),
            hidden_drop_keys: BTreeMap::new(),
            stage_pair_records: BTreeMap::new(),
            altar_enemy_ids: Default::default(),
            altar_rewards: BTreeMap::new(),
            label_texts: vec![None; UNITS + LABEL_SPARES],
            map_names: Default::default(),
            drop_item_rows: Default::default(),
            filter_stage_maps: Default::default(),
            slot_unlock_rows: [[0; 2]; 19],
            matatabi_rows: Default::default(),
            item_shop_rows: Default::default(),
            rank_gift_messages: Default::default(),
            warning1_texts: vec![Vec::new(); 0x35],
            warning2_rows: Default::default(),
            main_menu_rows: Default::default(),
            main_menu_row_settings: Default::default(),
            main_menu_text_settings: Default::default(),
            god_explanation: Default::default(),
            stage_first_messages: Default::default(),
            challenge_mode_texts: Default::default(),
            page_names: Default::default(),
            first_lose_texts: Default::default(),
            option_rows: Default::default(),
            main_menu_popups: Default::default(),
            tutorial_pages: Default::default(),
            popup_messages: Default::default(),
            category_explanations: Default::default(),
            stamp_messages: Default::default(),
            gift_messages: Default::default(),
            unit_evolve_rows: Default::default(),
            matatabi_popups: Default::default(),
            device_screen_w: 0,
            device_screen_h: 0,
            event_display: Default::default(),
            daily_login_rows: Default::default(),
            daily_login_groups: Default::default(),
            daily_login_max_id: 0,
            beacon_rows: Default::default(),
            beacon_groups: Default::default(),
            reward_stage_ids: Default::default(),
            lose_rows: Vec::new(),
            lose_row_settings: Vec::new(),
            lose_text_settings: Vec::new(),
            web_popup_pending: BTreeMap::new(),
            img004_sheet: None,
            reward_claimed: BTreeMap::new(),
            release_points: BTreeMap::new(),
            server_flags: BTreeMap::new(),
            drop_chara_max_1000: -1,
            drop_chara_max_1100: -1,
            img039_sheet: None,
            deck_button_x: [
                0x9f, 0x121, 0x1a3, 0x225, 0x2a7, 0xab, 0x12d, 0x1af, 0x231, 0x2b3,
            ],
            enemy_kill_counts: BTreeMap::new(),
            best_scores: BTreeMap::new(),
            ranking_entries: Vec::new(),
            boss_shockwave_anim: Maanim::default(),
            cannon_part_rows: Default::default(),
            ex_option_targets: Default::default(),
            ex_replacement_stages: Default::default(),
            built_deck_stages: Default::default(),
            built_deck_records: Default::default(),
            cannon_parts: Default::default(),
            enemy_castle: Vec::new(),
            fixed_lineup_store: Default::default(),
            combo_store: Default::default(),
            chara_groups: Default::default(),
            crown_multipliers: Default::default(),
            map_options: Default::default(),
            settings: Default::default(),
            failed_packs: BTreeSet::new(),
            entropy: Entropy::loose(),
            dojo_chest_rows: Vec::new(),
            gold_cpu_rows: Vec::new(),
            lock_skip_rows: Vec::new(),
            realms_rng_tables: BTreeMap::new(),
            point_event_rewards: BTreeMap::new(),
            event_gatya_items: BTreeMap::new(),
            leadership_return_maps: BTreeMap::new(),
            event_gatya_settings: BTreeMap::new(),
            orb_effects: Default::default(),
            catseye_behavior: BTreeMap::new(),
            ability_data_rows: [[0; 5]; 10],
            gatya_data_sets: BTreeMap::new(),
            gatya_chance_animations: Vec::new(),
            recommended_powerup_rows: Vec::new(),
            recommended_levelups: BTreeMap::new(),
            officers_club_rows: BTreeMap::new(),
            item_pack_rows: BTreeMap::new(),
            autoset_ratings: BTreeMap::new(),
            autoset_abilities: Vec::new(),
            autoset_excluded_enemies: Vec::new(),
            autoset_groups: BTreeMap::new(),
            autoset_organizations: BTreeMap::new(),
            map_layouts: BTreeMap::new(),
            vibration: VibrationStore::default(),
            daily_login_grades: BTreeMap::new(),
            sound_state: SoundState::default(),
            change_conditions: BTreeMap::new(),
            score_bonus_maps: BTreeMap::new(),
            hidden_data: HiddenData::default(),
            medals: Vec::new(),
            medal_order: Vec::new(),
            treasure_store: Default::default(),
            orb_store: Default::default(),
            special_rules: Default::default(),
            equipped_orbs: Default::default(),
            map_data_ids: Default::default(),
            map_stage_sets: Default::default(),
            talent_definitions: Default::default(),
            talent_levels: Default::default(),
            gamatoto_logs: Default::default(),
            gamatoto_member_stats: Default::default(),
            gamatoto_member_names: Default::default(),
            gamatoto_messages: Default::default(),
            gamatoto_stage_free: Default::default(),
            gamatoto_stage_names: Default::default(),
            gamatoto_stage_event_names: Default::default(),
            gamatoto_stage_rows: Default::default(),
            gamatoto_stage_event_rows: Default::default(),
            gamatoto_drop_rows: Default::default(),
            gamatoto_log_counts: [0; 3],
            gamatoto_limit: [0; 3],
            gamatoto_member_ids: [-1; 71],
            ad_reward_rows: Vec::new(),
            gamatoto_collabo: Default::default(),
            gamatoto_collabo_stages: Vec::new(),
            mission_gatya_settings: Default::default(),
            mission_limit_options: Default::default(),
            mission_unlock_conditions: Default::default(),
            mission_data: Default::default(),
            mission_names: Default::default(),
            mission_descriptions: Default::default(),
            mission_monthly: Default::default(),
            mission_groups: Default::default(),
            mission_condition_settings: Default::default(),
            mission_max_type1: 0,
            mission_max_type3: 0,
            mission_max_type2: 0,
            mission_max_type0: 0,
            map_stage_limit_messages: Default::default(),
            stage_names_numbered: Default::default(),

            stage_difficulty: Default::default(),
            treasure1_texts: Default::default(),
            treasure2_texts: Default::default(),
            treasure3_texts: Default::default(),
            treasure3_after_texts: Default::default(),
            stage_name_variants: Default::default(),
            castle_recipe_unlock_data: Default::default(),
            castle_recipe_unlocks: Default::default(),
            castle_mix_recipes: Default::default(),
            castle_recipe_names: Default::default(),
            castle_recipe_texts: Default::default(),
            castle_recipes: Default::default(),
            castle_custom_limit: 0,
            gamatoto_bonus: Default::default(),
            gamatoto_special_drops: Default::default(),
            gamatoto_unlocks: Default::default(),
            ending_messages: Vec::new(),
            opening_messages: Default::default(),
            map_stage_shortcuts: Default::default(),
            drop_chara_rows: Vec::new(),
            rank_gift_rows: Vec::new(),
            unit_limit_rows: Vec::new(),
            unit_limit_extra: [-1; 110],
            zombie_lottery: Default::default(),
            unlock_popup_rows: Default::default(),
            outbreak_active: Default::default(),
            outbreak_cleared: Default::default(),
            cleared_map_ids: Default::default(),
            stages_cleared_cache: Default::default(),
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
            dialog_sheet: Default::default(),
            scene_img008_sheet: Default::default(),
            guide_label: Default::default(),
            tutorial_lines: Default::default(),
            surface_lost: false,
            ui_sheet_cache: Default::default(),
            medals_awarded_flags: BTreeMap::new(),
            medals_pending: Vec::new(),
            club_user_rank: 0,
            club_pass_state: [0; 8],
            club_owned: BTreeMap::new(),
            trait_icons: Default::default(),
            ability_icons: Default::default(),
            enemy_book_rows: Default::default(),
            enemy_book_pages: Default::default(),
            enemy_book_question: Default::default(),
            cat_book_pages: Default::default(),
            cat_book_question: Default::default(),
            cat_book_rows: Default::default(),
            cat_book_data: Default::default(),
            enemy_dictionary_ids: Default::default(),
            enemy_dictionary_pages: Default::default(),
            enemy_dictionary_groups: Default::default(),
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
            miracle_levels: Default::default(),
            god_item_texts: vec![Default::default(); 4],
            god_item_names: vec![Vec::new(); 4],
            god_name_text: Default::default(),
            god_bought_texts: Default::default(),
            god_short_texts: Default::default(),
            god_chatter_texts: vec![Default::default(); 0x21],
            god_intro_texts: vec![Default::default(); 3],
            menu_texts: (0..0x3bc).map(|_| None).collect(),
            map_reopen_times: Default::default(),
            item_drop_queue: Default::default(),
            drop_icons: BTreeMap::new(),
            web_popup_entries: Default::default(),
            web_popup_shown: Default::default(),
            ad_button_id: Default::default(),
            ad_button_cleared: Default::default(),
            lineup_stages: Default::default(),
            lineup_records: Default::default(),
            labyrinth_units: Default::default(),
            labyrinth_units_used: Default::default(),
            labyrinth_floors: Default::default(),
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
            stage_unlock_cache: Default::default(),
            stage_record_neg20: Default::default(),
            stage_record_neg19: Default::default(),
            stage_record_neg18: Default::default(),
            stage_record_neg17: Default::default(),
            stage_record_neg16: Default::default(),
            stage_record_neg11: Default::default(),
            stage_record_neg10: Default::default(),
            stage_record_neg9: Default::default(),
            stage_record_neg4: Default::default(),
            stage_table_neg26: Default::default(),
            stage_table_neg24: Default::default(),
            stage_table_neg23: Default::default(),
            stage_table_neg22: Default::default(),
            map_flag_neg4: Default::default(),
            map_flag_neg9: Default::default(),
            map_flag_neg10: Default::default(),
            map_flag_neg11: Default::default(),
            map_flag_neg16: Default::default(),
            map_flag_neg17: Default::default(),
            map_flag_neg18: Default::default(),
            map_flag_neg20: Default::default(),
            map_count_neg4: Default::default(),
            map_count_neg10: Default::default(),
            map_count_neg11: Default::default(),
            map_count_neg16: Default::default(),
            map_count_neg17: Default::default(),
            map_count_neg18: Default::default(),
            map_count_neg20: Default::default(),
            map_count_neg21: Default::default(),
            map_stamp_neg4: Default::default(),
            map_stamp_neg10: Default::default(),
            map_stamp_neg11: Default::default(),
            map_stamp_neg16: Default::default(),
            map_stamp_neg17: Default::default(),
            map_stamp_neg18: Default::default(),
            map_stamp_neg20: Default::default(),
            map_stamp_neg21: Default::default(),
            map_stamp_3: Default::default(),
            map_stamp_4: Default::default(),
            map_opened_neg4: Default::default(),
            map_opened_neg10: Default::default(),
            map_opened_neg11: Default::default(),
            map_opened_neg16: Default::default(),
            map_opened_neg17: Default::default(),
            map_opened_neg18: Default::default(),
            map_opened_neg20: Default::default(),
            map_opened_neg21: Default::default(),
            map_opened_3: Default::default(),
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

    pub fn units_owned_key(&self) -> usize {
        Self::UNITS_OWNED.wrapping_add((self.limits.units as usize).wrapping_mul(4))
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
        self.rng_state
    }

    pub fn set_rng_state(&mut self, state: u32) {
        self.rng_state = state;
    }

    pub fn zero(&mut self, off: usize, len: usize) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..len))
            .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

        bytes.fill(0);

        Ok(())
    }

    pub fn block_at<const N: usize>(&self, off: usize) -> Result<[u8; N], Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..N))
                .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

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
            .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

        bytes.copy_from_slice(&value);

        Ok(())
    }

    pub fn bytes_from(&self, off: usize) -> Result<&[u8], Fault> {
        self.raw.get(off..).ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))
    }

    pub fn f32_at(&self, off: usize) -> Result<f32, Fault> {
        Ok(f32::from_le_bytes(self.block_at::<4>(off)?))
    }

    pub fn set_f32_at(&mut self, off: usize, value: f32) -> Result<(), Fault> {
        self.set_block_at::<4>(off, value.to_le_bytes())
    }

    pub fn u8_at(&self, off: usize) -> Result<u8, Fault> {
        self.raw.get(off).copied().ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))
    }

    pub fn i8_at(&self, off: usize) -> Result<i8, Fault> {
        self.raw
            .get(off)
            .map(|byte| *byte as i8)
            .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))
    }

    pub fn i16_at(&self, off: usize) -> Result<i16, Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..2))
                .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

        let mut word = [0u8; 2];
        word.copy_from_slice(bytes);

        Ok(i16::from_le_bytes(word))
    }

    pub fn i32_at(&self, off: usize) -> Result<i32, Fault> {
        let bytes =
            self.raw
                .get(off..)
                .and_then(|rest| rest.get(..4))
                .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

        let mut word = [0u8; 4];
        word.copy_from_slice(bytes);

        Ok(i32::from_le_bytes(word))
    }

    pub fn set_i32_at(&mut self, off: usize, value: i32) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..4))
            .ok_or(Fault::index_out_of_range(off as i64, SIZE as i64))?;

        bytes.copy_from_slice(&value.to_le_bytes());

        Ok(())
    }
}
