use std::collections::BTreeMap;

use crate::{
    Fault,
    engine::{
        AppContext, altar_recompute, load_map_stage_csv, map_index_of_map_id, map_type_as_index, map_type_of_map_id,
        texture_cache_load,
    },
};

pub const DECK_SLOTS: usize = 10;
pub const TECH_COUNT: usize = 0xb;
pub const TREASURE_CHAPTERS: usize = 0xa;
pub const TREASURE_STAGES: usize = 0x31;
pub const BATTLE_ITEMS: usize = 6;

const FREE_MAP_MODE: i32 = 3;
const EMPIRE_TYPE: i32 = -2;
const FUTURE_TYPE: i32 = -3;
const COSMOS_TYPE: i32 = -7;
const FUTURE_MODE: i32 = 4;
const COSMOS_MODE: i32 = 7;
const EXTRA_TYPE: i32 = -8;
const EXTRA_MODE: i32 = 0x63;
const BATTLE_INTRO_START: i32 = 0x726;
const SCORED_TYPES: [i32; 3] = [3, 4, -24];
const CAT_SIDE: i32 = 1;
const ENEMY_SIDE: i32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StageEntry {
    pub map_id: i32,
    pub stage: i32,
    pub crown: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SetupUnit {
    pub unit: i32,
    pub form: i32,
    pub level: u32,
    pub plus: u32,
    pub talents: Vec<(i32, i32)>,
    pub orbs: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechLevel {
    pub level: u32,
    pub plus: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartLevels {
    pub cannon: i32,
    pub foundation: i32,
    pub style: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CatGod {
    Absent,
    #[default]
    Present,
    Discounted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setup {
    pub stage: StageEntry,
    pub lineup: Vec<SetupUnit>,
    pub tech: [TechLevel; TECH_COUNT],
    pub treasures: [[i32; TREASURE_STAGES]; TREASURE_CHAPTERS],
    pub cannon: i32,
    pub style: i32,
    pub foundation: i32,
    pub parts: BTreeMap<i32, PartLevels>,
    pub items: [bool; BATTLE_ITEMS],
    pub altar: Option<i32>,
    pub cat_god: CatGod,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            stage: StageEntry::default(),
            lineup: Vec::new(),
            tech: [TechLevel::default(); TECH_COUNT],
            treasures: [[0; TREASURE_STAGES]; TREASURE_CHAPTERS],
            cannon: 0,
            style: 0,
            foundation: 0,
            parts: BTreeMap::new(),
            items: [true; BATTLE_ITEMS],
            altar: None,
            cat_god: CatGod::default(),
        }
    }
}

pub fn select_stage(ctx: &mut AppContext, entry: StageEntry) -> Result<(), Fault> {
    let map_type = map_type_of_map_id(entry.map_id);
    let index = map_index_of_map_id(entry.map_id);

    let mode = match map_type {
        EMPIRE_TYPE => index,
        FUTURE_TYPE => index.wrapping_add(FUTURE_MODE),
        COSMOS_TYPE => index.wrapping_add(COSMOS_MODE),
        EXTRA_TYPE => EXTRA_MODE,
        _ => FREE_MAP_MODE,
    };

    ctx.set_i32_at(AppContext::CHAPTER_MODE, mode)?;
    ctx.set_block_at::<1>(AppContext::SCORE_MODE_FLAG, [u8::from(SCORED_TYPES.contains(&map_type))])?;

    if mode == FREE_MAP_MODE {
        ctx.set_i32_at(AppContext::SAVED_MAP_TYPE, map_type_as_index(map_type))?;
        ctx.set_i32_at(AppContext::MAP_INDEX, index)?;
        ctx.set_i32_at(AppContext::CROWN_LEVEL, entry.crown)?;
    }

    if mode == EXTRA_MODE {
        ctx.set_i32_at(AppContext::OUTRO_CHAPTER_MODE, FREE_MAP_MODE)?;
        ctx.set_i32_at(AppContext::OUTRO_ENTRY_STAGE, 0)?;
        ctx.set_i32_at(AppContext::EX_MAP, index)?;
        ctx.set_i32_at(AppContext::EX_STAGE, entry.stage)?;
    }

    ctx.set_i32_at(AppContext::ENTRY_STAGE, entry.stage)?;
    ctx.set_i32_at(if mode == EXTRA_MODE { AppContext::EX_STAGE_INDEX } else { AppContext::STAGE_INDEX }, entry.stage)?;
    ctx.set_i32_at(AppContext::faction_flags(0), CAT_SIDE)?;
    ctx.set_i32_at(AppContext::faction_flags(1), ENEMY_SIDE)
}

pub fn is_extra_entry(ctx: &AppContext) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::CHAPTER_MODE)? == EXTRA_MODE)
}

pub fn prepare_extra_entry(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::BATTLE_ENTRY_RESET, 0)?;
    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
    ctx.set_block_at::<1>(AppContext::CURTAIN_STYLE, [1])?;
    altar_recompute(ctx)?;
    texture_cache_load(ctx, b"img015.png", b"img015.imgcut", 0x2601)?;
    ctx.set_i32_at(AppContext::BATTLE_INTRO_FRAME, BATTLE_INTRO_START)?;
    ctx.set_i32_at(AppContext::EVENT_POINT_BOOST, 1)?;
    ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
    ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;

    let map = ctx.i32_at(AppContext::EX_MAP)?;

    load_map_stage_csv(ctx, map, 0, 1, 0, 1, 1)
}
