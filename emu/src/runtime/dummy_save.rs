use crate::{
    Fault,
    engine::{AppContext, obfuscate_value},
};

use super::fill_dummy_lineup;

const MEDAL_PROGRESS_CAP: i32 = 2_000_000_000;
const STORY_CHAPTERS: usize = 5;
const MAPS_PER_TYPE: usize = 0x1f4;
const STAGES_PER_MAP: usize = 0x30;
const STARS: usize = 4;

const TUTORIAL_DONE: i32 = 1;
const TECH_COUNT: usize = 0xb;
const TECH_LEVEL: u32 = 0x14;
const PLUS_LEVEL: u32 = 0xa;
const COMBO_UNLOCKED: i32 = 0;
const PLUS_SHIFT: u32 = 0x10;
const CAT_FOOD: u32 = 45_000;
const ZOOM_ANCHOR: i32 = 0x208;
const CHAPTER_ROWS: usize = 0xa;
const CHAPTER_ROW_STRIDE: usize = 0xd0;
const CHAPTER_STAGES: usize = 0x30;
const STAGE_CLEARED: i32 = 1;
const TREASURE_CHAPTERS: usize = 0xa;
const TREASURE_STAGES: usize = 0x31;
const SUPERIOR_TREASURE: i32 = 3;
const SPEED_MODES: usize = 3;
const TUTORIALS_SEEN: [usize; 4] = [
    AppContext::TUTORIAL_DECK_SEEN,
    AppContext::TUTORIAL_TWO_ROWS_SEEN,
    AppContext::TUTORIAL_CAT_GOD_SEEN,
    AppContext::SHOP_TUTORIAL_SEEN,
];

pub fn fill_dummy_save(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::TUTORIAL_CLEARED, TUTORIAL_DONE)?;

    for seen in TUTORIALS_SEEN {
        ctx.set_i32_at(seen, TUTORIAL_DONE)?;
    }

    ctx.set_i32_at(AppContext::MEDAL_MONEY_0, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_1, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_4, MEDAL_PROGRESS_CAP)?;

    ctx.set_i32_at(AppContext::BATTLE_ZOOM_Y, ZOOM_ANCHOR)?;

    for chapter in 0..CHAPTER_ROWS {
        for stage in 0..CHAPTER_STAGES {
            ctx.set_i32_at(
                AppContext::STAGE_RECORD_CHAPTERS + chapter * CHAPTER_ROW_STRIDE + stage * 4,
                STAGE_CLEARED,
            )?;
        }
    }

    for tech in 0..TECH_COUNT {
        let mut cell = [0u8; 8];

        cell[..4].copy_from_slice(&(PLUS_LEVEL << PLUS_SHIFT | TECH_LEVEL).to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::TECH_LEVELS + tech * 8, cell)?;
    }

    for chapter in 0..TREASURE_CHAPTERS {
        for stage in 0..TREASURE_STAGES {
            ctx.set_i32_at(
                AppContext::TREASURE_LEVELS + chapter * AppContext::TREASURE_LEVELS_STRIDE + stage * 4,
                SUPERIOR_TREASURE,
            )?;
        }
    }

    for mode in 0..SPEED_MODES {
        ctx.set_block_at::<1>(AppContext::POWERUP_AVAILABLE + mode, [1])?;
    }

    let mut food = [0u8; 8];

    food[..4].copy_from_slice(&CAT_FOOD.to_le_bytes());
    obfuscate_value(&mut food);
    ctx.set_block_at(AppContext::ITEM_16_COUNT, food)?;

    ctx.set_i32_at(AppContext::SELECTED_DECK_PRESET, 0)?;

    fill_dummy_lineup(ctx)?;

    for chapter in 0..STORY_CHAPTERS {
        ctx.set_i32_at(
            AppContext::STORY_MAP_COUNTS + chapter * 4,
            MAPS_PER_TYPE as i32,
        )?;
    }

    for maps in [
        &mut ctx.maps_neg26,
        &mut ctx.maps_neg24,
        &mut ctx.maps_neg23,
        &mut ctx.maps_neg22,
        &mut ctx.maps_neg21,
        &mut ctx.maps_neg20,
        &mut ctx.maps_neg19,
        &mut ctx.maps_neg18,
        &mut ctx.maps_neg17,
        &mut ctx.maps_neg16,
        &mut ctx.maps_neg11,
        &mut ctx.maps_neg10,
        &mut ctx.maps_neg9,
        &mut ctx.maps_neg4,
    ] {
        maps.resize(MAPS_PER_TYPE, [0; 3]);
    }

    for nested in [
        &mut ctx.stage_record_neg26,
        &mut ctx.stage_record_neg24,
        &mut ctx.stage_record_neg23,
        &mut ctx.stage_record_neg22,
    ] {
        nested.resize(MAPS_PER_TYPE, vec![vec![0i16; STAGES_PER_MAP]; STARS]);
    }

    for flat in [
        &mut ctx.stage_record_neg20,
        &mut ctx.stage_record_neg19,
        &mut ctx.stage_record_neg18,
        &mut ctx.stage_record_neg17,
        &mut ctx.stage_record_neg16,
        &mut ctx.stage_record_neg11,
    ] {
        flat.resize(MAPS_PER_TYPE * STAGES_PER_MAP, 0);
    }

    for wide in [
        &mut ctx.stage_record_neg10,
        &mut ctx.stage_record_neg9,
        &mut ctx.stage_record_neg4,
    ] {
        wide.resize(MAPS_PER_TYPE * STAGES_PER_MAP, 0);
    }

    for nested in [
        &mut ctx.stage_unlock_neg26,
        &mut ctx.stage_unlock_neg24,
        &mut ctx.stage_unlock_neg23,
        &mut ctx.stage_unlock_neg22,
    ] {
        nested.resize(MAPS_PER_TYPE, vec![0i8; STARS]);
    }

    for flat in [
        &mut ctx.stage_unlock_neg20,
        &mut ctx.stage_unlock_neg19,
        &mut ctx.stage_unlock_neg18,
        &mut ctx.stage_unlock_neg17,
        &mut ctx.stage_unlock_neg16,
        &mut ctx.stage_unlock_neg11,
    ] {
        flat.resize(MAPS_PER_TYPE * STARS, 0);
    }

    for nested in [
        &mut ctx.stages_cleared_neg26,
        &mut ctx.stages_cleared_neg24,
        &mut ctx.stages_cleared_neg23,
        &mut ctx.stages_cleared_neg22,
    ] {
        nested.resize(MAPS_PER_TYPE, vec![0i8; STARS]);
    }

    for flat in [
        &mut ctx.stages_cleared_neg20,
        &mut ctx.stages_cleared_neg18,
        &mut ctx.stages_cleared_neg17,
        &mut ctx.stages_cleared_neg16,
        &mut ctx.stages_cleared_neg11,
    ] {
        flat.resize(MAPS_PER_TYPE * STARS, 0);
    }

    for wide in [
        &mut ctx.stages_cleared_neg10,
        &mut ctx.stages_cleared_neg9,
        &mut ctx.stages_cleared_neg4,
    ] {
        wide.resize(MAPS_PER_TYPE * STARS, 0);
    }

    for wide in [
        &mut ctx.stage_unlock_neg10,
        &mut ctx.stage_unlock_neg9,
        &mut ctx.stage_unlock_neg4,
    ] {
        wide.resize(MAPS_PER_TYPE * STARS, 0);
    }

    Ok(())
}

pub fn unlock_dummy_combos(ctx: &mut AppContext) {
    ctx.combo_store.states.fill(COMBO_UNLOCKED);
}
