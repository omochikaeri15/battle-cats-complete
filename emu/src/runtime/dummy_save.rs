use crate::{Fault, engine::AppContext};

const MEDAL_PROGRESS_CAP: i32 = 2_000_000_000;
const STORY_CHAPTERS: usize = 5;
const MAPS_PER_TYPE: usize = 0x1f4;
const STAGES_PER_MAP: usize = 0x30;
const STARS: usize = 4;
const DECK_SLOTS: usize = 10;

const LINEUP: [i32; DECK_SLOTS] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const TUTORIAL_DONE: i32 = 1;
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

    ctx.set_i32_at(AppContext::SELECTED_DECK_PRESET, 0)?;

    for (slot, unit) in LINEUP.iter().enumerate() {
        ctx.set_i32_at(AppContext::DECK_PRESETS + slot * 4, unit.wrapping_add(2))?;
    }

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

    for wide in [
        &mut ctx.stage_unlock_neg10,
        &mut ctx.stage_unlock_neg9,
        &mut ctx.stage_unlock_neg4,
    ] {
        wide.resize(MAPS_PER_TYPE * STARS, 0);
    }

    Ok(())
}
