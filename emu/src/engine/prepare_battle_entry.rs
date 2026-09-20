use crate::Fault;

use super::{
    AppContext, altar_recompute, get_background_id, get_global_map_id, get_map_type,
    get_stage_index, is_score_stage, load_battle_snapshot, load_legend_quest_csvs,
    load_map_stage_csv, load_stage_csv, select_point_group, select_point_map, set_point_stage,
    setup_bg_color, texture_cache_load,
};

const BATTLE_INTRO_START: i32 = 0x726;

pub fn prepare_battle_entry(ctx: &mut AppContext) -> Result<bool, Fault> {
    ctx.set_i32_at(AppContext::BATTLE_ENTRY_RESET, 0)?;
    ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
    ctx.set_block_at::<1>(AppContext::CURTAIN_STYLE, [1])?;
    altar_recompute(ctx)?;
    load_battle_snapshot(ctx, 0)?;
    load_legend_quest_csvs(ctx)?;

    let background = get_background_id(ctx)?;

    setup_bg_color(ctx, background)?;

    let map_id = get_global_map_id(ctx, 0)?;

    select_point_map(ctx, map_id)?;

    if is_score_stage(ctx.event_items.as_ref()) {
        let map_id = get_global_map_id(ctx, 0)?;

        select_point_group(ctx, map_id)?;

        let stage = get_stage_index(ctx)?;

        set_point_stage(ctx, stage)?;
    }

    texture_cache_load(ctx, b"img015.png", b"img015.imgcut", 0x2601)?;
    ctx.set_i32_at(AppContext::BATTLE_INTRO_FRAME, BATTLE_INTRO_START)?;

    let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if mode > 2 {
        ctx.set_i32_at(AppContext::EVENT_POINT_BOOST, 1)?;

        if mode == 3 {
            let map = ctx.i32_at(AppContext::MAP_INDEX)?;

            if !load_map_stage_csv(ctx, map, 0, 1, 0, 0, 1)? {
                return Ok(false);
            }

            let stage = ctx.i32_at(AppContext::STAGE_ROW)?;

            load_stage_csv(ctx, stage, 1)?;

            return Ok(true);
        }
    } else {
        ctx.set_i32_at(AppContext::EVENT_POINT_BOOST, mode)?;
    }

    let map_type = get_map_type(ctx, 0)?;
    let (chapter, map) = if map_type == -2 || map_type == -0xc {
        (0, ctx.i32_at(AppContext::CHAPTER_MODE)?)
    } else if map_type == -3 || map_type == -0xd {
        (1, ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_sub(4))
    } else if map_type == -7 || map_type == -0xe {
        (2, ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_sub(7))
    } else {
        (0, 0)
    };

    load_map_stage_csv(ctx, chapter, map, 1, 1, 0, 1)
}
