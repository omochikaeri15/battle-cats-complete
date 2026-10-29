use crate::{
    Fault,
    engine::{
        AltarReward, AppContext, get_map_count, get_stage_count, map_index_of_map_id, map_type_base_id, map_type_of_map_id,
        obfuscate_value, reset_stage_name_tables, set_stage_record,
    },
};

use super::{CatGod, Setup, StageEntry, StageVariant, fill_dummy_lineup};

const MEDAL_PROGRESS_CAP: i32 = 2_000_000_000;
const POINT_CAP: i32 = 9_999_999;
const LEGEND_TYPE: i32 = -8;
const LEGEND_MAP_STRIDE: i64 = 0x30;
const ALTAR_KEY_STAGES: i32 = 100;

const TUTORIAL_DONE: i32 = 1;
const MIRACLE_PRICES: [u32; 4] = [20, 10, 5, 90];
const CAT_GOD_INTRO_DONE: i32 = 4;
const DISCOUNT_CHAPTER: usize = 7;
const CHAPTER_CLEARED: u32 = 0x30;
const COMBO_UNLOCKED: i32 = 0;
const BASE_SHIFT: u32 = 0x10;
const CAT_FOOD: u32 = 45_000;
const ZOOM_ANCHOR: i32 = 0x208;
const CHAPTER_ROWS: usize = 0xa;
const CHAPTER_ROW_STRIDE: usize = 0xd0;
const CHAPTER_STAGES: usize = 0x30;
const STAGE_CLEARED: i32 = 1;
const TWO_ROWS_UNLOCKED: i32 = 2;
const MAP_RETURN_POPUP: i32 = 0x4b;
const MOON_SLOT: i32 = 0x2f;
const MOON_CHAPTERS: [i32; 9] = [0, 1, 2, 4, 5, 6, 7, 8, 9];
const ZERO_LEGENDS_TYPE: i32 = -22;
const LEGENDS_TYPE: i32 = 0;
const UNCANNY_TYPE: i32 = -9;
const AKU_TYPE: i32 = -19;
const RELICS_MAP: i32 = 0x30;
const SACRED_FOREST_MAP: i32 = 0x30;
const ANCIENT_FOREST_MAP: i32 = 0x23;
const AKU_MAP: i32 = 0;
const ELDRITCH_EX_MAP: i32 = 0x1a;
const ORIGIN_EX_MAP: i32 = 0x45;
const FACE_EX_MAP: i32 = 0x44;
const AKU_EX_MAP: i32 = 0x2a;

pub fn fill_dummy_save(ctx: &mut AppContext, setup: &Setup) -> Result<(), Fault> {
    let tutorial = &setup.tutorial;

    for (seen, done) in [
        (AppContext::TUTORIAL_CLEARED, tutorial.battle_cleared),
        (AppContext::TUTORIAL_DECK_SEEN, tutorial.deck_seen),
        (AppContext::TUTORIAL_CAT_GOD_SEEN, tutorial.cat_god_seen),
        (AppContext::SHOP_TUTORIAL_SEEN, tutorial.shop_seen),
    ] {
        ctx.set_i32_at(seen, if done { TUTORIAL_DONE } else { 0 })?;
    }

    ctx.set_i32_at(AppContext::TUTORIAL_TWO_ROWS_SEEN, if tutorial.two_rows_seen { TWO_ROWS_UNLOCKED } else { 0 })?;

    ctx.set_i32_at(AppContext::MEDAL_MONEY_0, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_1, MEDAL_PROGRESS_CAP)?;
    ctx.set_i32_at(AppContext::MEDAL_MONEY_4, MEDAL_PROGRESS_CAP)?;
    ctx.unlock_popups.insert(MAP_RETURN_POPUP, true);

    ctx.set_i32_at(AppContext::BATTLE_ZOOM_Y, ZOOM_ANCHOR)?;
    reset_stage_name_tables(ctx)?;

    for chapter in 0..CHAPTER_ROWS {
        for stage in 0..CHAPTER_STAGES {
            ctx.set_i32_at(
                AppContext::STAGE_RECORD_CHAPTERS + chapter * CHAPTER_ROW_STRIDE + stage * 4,
                STAGE_CLEARED,
            )?;
        }
    }

    for (tech, held) in setup.tech.iter().enumerate() {
        let mut cell = [0u8; 8];
        let packed = held.level.saturating_sub(1) << BASE_SHIFT | held.plus;

        cell[..4].copy_from_slice(&packed.to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::TECH_LEVELS + tech * 8, cell)?;
    }

    for (chapter, stages) in setup.treasures.iter().enumerate() {
        for (stage, level) in stages.iter().enumerate() {
            ctx.set_i32_at(
                AppContext::TREASURE_LEVELS + chapter * AppContext::TREASURE_LEVELS_STRIDE + stage * 4,
                *level,
            )?;
        }
    }

    let preset = AppContext::PRESET_CANNON_PARTS;

    ctx.set_block_at::<3>(
        preset,
        [setup.cannon as u8, setup.style as u8, setup.foundation as u8],
    )?;

    let mut food = [0u8; 8];

    food[..4].copy_from_slice(&CAT_FOOD.to_le_bytes());
    obfuscate_value(&mut food);
    ctx.set_block_at(AppContext::ITEM_16_COUNT, food)?;

    ctx.set_i32_at(AppContext::SELECTED_DECK_PRESET, 0)?;

    fill_dummy_lineup(ctx, setup)?;

    Ok(())
}

pub fn fill_dummy_cannon_parts(ctx: &mut AppContext, setup: &Setup) {
    for (part, levels) in &setup.parts {
        ctx.cannon_part_rows.insert(
            *part,
            vec![0, levels.cannon.wrapping_sub(1), levels.foundation, levels.style],
        );
    }
}

pub fn seed_cat_god(ctx: &mut AppContext, setup: &Setup) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::CAT_GOD_AVAILABLE, i32::from(setup.cat_god != CatGod::Absent))?;
    ctx.set_i32_at(AppContext::CAT_GOD_INTRO_STEP, CAT_GOD_INTRO_DONE)?;

    for (miracle, price) in MIRACLE_PRICES.into_iter().enumerate() {
        let mut cell = [0u8; 8];

        cell[..4].copy_from_slice(&price.to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::MIRACLE_PRICES + miracle * 8, cell)?;
    }

    if setup.cat_god == CatGod::Discounted {
        let key = u32::from_le_bytes(ctx.block_at::<4>(AppContext::STAGES_CLEARED_CHAPTERS_KEY)?);

        ctx.set_block_at(AppContext::STAGES_CLEARED_CHAPTERS + DISCOUNT_CHAPTER * 4, (CHAPTER_CLEARED ^ key).to_le_bytes())?;
    }

    Ok(())
}

pub fn seed_stage_variant(ctx: &mut AppContext, entry: StageEntry) -> Result<(), Fault> {
    let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let stage = ctx.i32_at(AppContext::ENTRY_STAGE)?;

    match entry.variant {
        StageVariant::Plain => {}
        StageVariant::Outbreak => {
            ctx.outbreak_active.entry(mode).or_default().insert(stage, true);
        }
        StageVariant::Invasion => {
            let key = u32::from_le_bytes(ctx.block_at::<4>(AppContext::STAGES_CLEARED_CHAPTERS_KEY)?);

            ctx.set_block_at(AppContext::STAGES_CLEARED_CHAPTERS + mode as usize * 4, (CHAPTER_CLEARED ^ key).to_le_bytes())?;
            ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [stage as u8])?;
        }
        StageVariant::ZombieInvasion => {
            ctx.set_block_at::<1>(AppContext::MAP_NEG15_CLEARED, [1])?;
            ctx.cleared_map_ids.push(map_type_base_id(ZERO_LEGENDS_TYPE, 0));

            for chapter in MOON_CHAPTERS {
                ctx.outbreak_cleared.entry(chapter).or_default().insert(MOON_SLOT, true);
            }

            ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [stage as u8])?;
        }
        StageVariant::ExInvasion => {
            let map_type = map_type_of_map_id(entry.map_id);
            let map_index = map_index_of_map_id(entry.map_id);

            match (map_type, map_index) {
                (LEGENDS_TYPE, RELICS_MAP) => {
                    ctx.cleared_map_ids.push(map_type_base_id(map_type, map_index));
                    ctx.set_block_at::<1>(AppContext::EX_REDIRECT_A_BLOCKED, [0])?;
                }
                (UNCANNY_TYPE, SACRED_FOREST_MAP) => {
                    ctx.cleared_map_ids.push(map_type_base_id(map_type, map_index));
                    ctx.set_i32_at(ex_record(ORIGIN_EX_MAP), 0)?;
                }
                (UNCANNY_TYPE, ANCIENT_FOREST_MAP) => {
                    for map in 0..get_map_count(ctx, UNCANNY_TYPE)? {
                        ctx.cleared_map_ids.push(map_type_base_id(UNCANNY_TYPE, map));
                    }

                    ctx.set_block_at::<1>(AppContext::EX_REDIRECT_A_BLOCKED, [1])?;
                    ctx.set_i32_at(ex_record(ELDRITCH_EX_MAP), 1)?;
                    ctx.set_i32_at(ex_record(FACE_EX_MAP), 0)?;
                }
                (AKU_TYPE, AKU_MAP) => {
                    for cleared in 0..get_stage_count(ctx, AKU_TYPE, AKU_MAP)? {
                        set_stage_record(ctx, AKU_TYPE, AKU_MAP, cleared, 0, 1, 0)?;
                    }

                    ctx.set_i32_at(ex_record(AKU_EX_MAP), 0)?;
                }
                _ => ctx.ex_replacement_stages.entry(entry.map_id).or_default().push(stage),
            }
        }
    }

    Ok(())
}

fn ex_record(ex_map: i32) -> usize {
    (i64::from(ex_map) * LEGEND_MAP_STRIDE + AppContext::STAGE_RECORD_NEG8 as i64) as usize
}

pub fn seed_point_cap(ctx: &mut AppContext) {
    ctx.event_items.get_or_insert_with(Default::default).point_cap = POINT_CAP;
}

pub fn seed_altar_records(ctx: &mut AppContext, setup: &Setup) -> Result<(), Fault> {
    let rewards: Vec<(i32, AltarReward)> = ctx.altar_rewards.iter().map(|(key, reward)| (*key, *reward)).collect();
    let mut budget = setup.altar.map_or(0, |level| level.saturating_sub(1));

    for (key, reward) in rewards {
        let cleared = setup.altar.is_none() || (reward.unseal == 0 && reward.amount <= budget);

        if cleared && setup.altar.is_some() {
            budget -= reward.amount;
        }

        let map = key / ALTAR_KEY_STAGES;
        let stage = key % ALTAR_KEY_STAGES;
        let map_type = map_type_of_map_id(map);
        let map_index = map_index_of_map_id(map);

        if map_type == LEGEND_TYPE {
            let cell = i64::from(map_index) * LEGEND_MAP_STRIDE + i64::from(stage) * 4 + AppContext::STAGE_RECORD_NEG8 as i64;

            ctx.set_i32_at(cell as usize, i32::from(cleared))?;

            continue;
        }

        set_stage_record(ctx, map_type, map_index, stage, 0, i32::from(cleared), 0)?;
    }

    Ok(())
}

pub fn unlock_dummy_combos(ctx: &mut AppContext) {
    ctx.combo_store.states.fill(COMBO_UNLOCKED);
}
