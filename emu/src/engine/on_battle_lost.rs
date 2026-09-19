use crate::{operation, Fault};

use super::{
    ad_prepare, add_resource, add_stage_record, breadcrumb, call_rng, check_medals, commit_stage_score, defeat_counter_bump, entry_find_by_id,
    event_unit_slot, get_bottom_inset_logical, get_drawable_width, get_global_map_id, get_map_index, get_map_type, get_point_cap, get_point_id,
    get_point_rewards, get_point_total, get_release_point_cap, get_stage_best_score, get_stage_index, get_stage_record, get_stage_score,
    is_reward_claimed, is_score_stage, labyrinth_active, log_analytics_event, map_type_base_id, mission_progress, play_sound, point_reward_analytics,
    record_stage_played, scored_map_pays_money, set_auto_camera_mode, set_battle_status, set_fever_fade_out, set_reward_claimed, set_stage_record,
    sound_manager, validate_map_type, vibration_reset, web_popup_request, AppContext, Entity, ENTITY_BASE, ENTITY_STRIDE,
};

const SITE: &str = "on_battle_lost";

pub fn on_battle_lost(ctx: &mut AppContext) -> Result<(), Fault> {
    breadcrumb(ctx, 0x49)?;
    ctx.set_block_at::<2>(AppContext::RESULT_VIDEO_BUTTON, [0; 2])?;

    let map_type = get_map_type(ctx, 0)?;
    let map_index = get_map_index(ctx, 0)?;
    let stage = get_stage_index(ctx)?;

    ctx.set_i32_at(AppContext::LOST_MAP_TYPE, map_type)?;
    ctx.set_i32_at(AppContext::LOST_MAP_INDEX, map_index)?;
    ctx.set_i32_at(AppContext::LOST_STAGE, stage)?;

    if is_score_stage(ctx.event_items.as_ref()) {
        let store = ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?;
        let total = get_point_total(store);

        ctx.set_block_at::<1>(AppContext::POINT_LIMIT_PENDING, [(get_stage_score(store).wrapping_add(total) > get_point_cap(store)) as u8])?;

        let score = get_stage_score(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?);
        let stage = get_stage_index(ctx)?;
        let best = get_stage_best_score(ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?, stage);

        ctx.set_i32_at(AppContext::NEW_BEST_SCORE, (score > best) as i32)?;
        commit_stage_score(ctx)?;
        set_fever_fade_out(&mut ctx.special_rules);
    }

    set_auto_camera_mode(ctx, 1)?;
    sound_manager(ctx)?.stop_audio(-1);
    vibration_reset(ctx)?;

    if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 && !labyrinth_active(ctx)? && get_map_type(ctx, 0)? != -24 {
        let sound = if scored_map_pays_money(ctx)? { 0xbd } else { 0x39 };

        play_sound(sound_manager(ctx)?, sound, None);
    } else {
        let map_id = get_global_map_id(ctx, 0)?;
        let mut sound = 9;
        let mut silent = false;

        if ctx.map_records.contains_key(&map_id) {
            let map_id = get_global_map_id(ctx, 0)?;

            if !ctx.map_records.entry(map_id).or_default().defeat_voices.is_empty() {
                let map_id = get_global_map_id(ctx, 0)?;
                let mode = ctx.map_records.entry(map_id).or_default().defeat_voice_mode;
                let map_id = get_global_map_id(ctx, 0)?;

                if mode != 0 && ctx.map_records.entry(map_id).or_default().defeat_voice_mode == 1 {
                    let map_id = get_global_map_id(ctx, 0)?;
                    let mut index = 0usize;

                    while index < ctx.map_records.entry(map_id).or_default().defeat_voices.len() {
                        let voice = *ctx.map_records.entry(map_id).or_default().defeat_voices.get(index).ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: 0 })?;

                        if voice != -1 {
                            play_sound(sound_manager(ctx)?, voice, None);
                        }

                        index += 1;
                    }

                    silent = true;
                } else {
                    let count = ctx.map_records.entry(map_id).or_default().defeat_voices.len() as u32;
                    let map_id = get_global_map_id(ctx, 0)?;
                    let pick = call_rng(ctx, count as i32) as i64;
                    let voices = &ctx.map_records.entry(map_id).or_default().defeat_voices;

                    if (voices.len() as u64) > pick as u64 {
                        sound = *voices.get(pick as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: pick, limit: voices.len() as i64 })?;
                        silent = sound == -1;
                    }
                }
            }
        }

        if !silent {
            play_sound(sound_manager(ctx)?, sound, None);
        }

        defeat_counter_bump(ctx)?;
    }

    for slot in 1..=50usize {
        let entity = ENTITY_BASE + slot * ENTITY_STRIDE;

        if ctx.i32_at(entity + Entity::OCCUPANT)? == 0 || ctx.i32_at(entity + Entity::STATE)? == 4 {
            continue;
        }

        ctx.set_block_at::<8>(entity + Entity::STATE, 4u64.to_le_bytes())?;
        ctx.set_i32_at(entity + Entity::HP, 0)?;
        let speed = call_rng(ctx, 0xc).wrapping_mul(10).wrapping_add(100);

        ctx.set_i32_at(entity + Entity::SPEED, speed)?;

        let fall = 1000i32.wrapping_sub(call_rng(ctx, 0x28).wrapping_mul(10));

        ctx.set_i32_at(entity + Entity::DEFEAT_FALL, fall)?;
        ctx.set_i32_at(entity + Entity::DEFEAT_ALPHA, 0xff)?;

        let drift = operation::div_10(ctx.i32_at(entity + Entity::SPEED)?);
        let drift = call_rng(ctx, 0xa).wrapping_add(drift);

        ctx.set_i32_at(entity + Entity::DEFEAT_DRIFT, drift)?;

        let spin = call_rng(ctx, 0x28).wrapping_add(0x14);

        ctx.set_i32_at(entity + Entity::DEFEAT_SPIN, spin)?;

        let bounce = call_rng(ctx, 0xa).wrapping_add(5);

        ctx.set_i32_at(entity + Entity::DEFEAT_BOUNCE, bounce)?;
        ctx.set_i32_at(entity + Entity::DEFEAT_TIMER, 0)?;
    }

    set_battle_status(ctx, 2)?;
    ctx.deploy_queue.clear();
    ctx.set_i32_at(AppContext::LOSE_TIP_SHOWN, 0)?;

    if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 && !labyrinth_active(ctx)? && get_map_type(ctx, 0)? != -24 {
        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
        let played = get_stage_record(ctx, map_type, map_index, stage_row, star, 0)?;

        log_analytics_event(ctx, 0x1d, (played <= 0) as i32, 0, 0, 0)?;

        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

        add_stage_record(ctx, map_type, map_index, stage_row, star, 1, 0)?;

        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
        let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

        if get_stage_record(ctx, map_type, map_index, stage_row, star, 0)? >= 0x2710 {
            let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
            let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
            let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
            let star = ctx.i32_at(AppContext::STAR_LEVEL)?;

            set_stage_record(ctx, map_type, map_index, stage_row, star, 0x270f, 0)?;
        }

        record_stage_played(ctx)?;
        set_battle_status(ctx, 4)?;

        let map_id = get_global_map_id(ctx, 0)?;

        mission_progress(ctx, 0, map_id, 1, 0, 0)?;

        let stage_key = map_id.wrapping_mul(0x64);
        let row = ctx.i32_at(AppContext::STAGE_ROW)?.wrapping_add(stage_key);

        mission_progress(ctx, 1, row, 1, 0, 0)?;
        mission_progress(ctx, 9, operation::div_1000(map_id), 1, 0, 0)?;

        let row = stage_key.wrapping_add(ctx.i32_at(AppContext::STAGE_ROW)?);
        let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

        mission_progress(ctx, 0x15, row, total, 0, 0)?;

        if get_map_type(ctx, 0)? == 4 {
            let row = map_type_base_id(4, ctx.i32_at(AppContext::MAP_INDEX)?).wrapping_mul(0x64).wrapping_add(ctx.i32_at(AppContext::STAGE_ROW)?);
            let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

            mission_progress(ctx, 0x15, row, total, 0, 0)?;

            if entry_find_by_id(&ctx.ranking_entries, ctx.i32_at(AppContext::MAP_INDEX)?) < ctx.i32_at(AppContext::SCORE_TOTAL)? {
                ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 1)?;
            }
        } else {
            let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
            let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;
            let best = *ctx.best_scores.entry(map_index).or_default().entry(stage_row).or_insert(0);
            let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

            if best < total {
                let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
                let stage_row = ctx.i32_at(AppContext::STAGE_ROW)?;

                *ctx.best_scores.entry(map_index).or_default().entry(stage_row).or_insert(0) = total;
                ctx.set_i32_at(AppContext::NEW_BEST_SCORE, 1)?;
            }
        }
    } else if is_score_stage(ctx.event_items.as_ref()) {
        set_battle_status(ctx, 7)?;
        record_stage_played(ctx)?;

        let store = ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?;
        let point_id = get_point_id(store);
        let total = get_point_total(store);
        let rewards: Vec<(i32, i32, i32, i32, i32)> = get_point_rewards(&ctx.reward_defs, point_id)
            .map(|list| list.iter().map(|reward| (reward.id, reward.threshold, reward.kind, reward.target, reward.amount)).collect())
            .unwrap_or_default();
        let cap = get_release_point_cap(ctx, point_id)?;

        for (id, threshold, kind, target, amount) in rewards {
            if total < threshold || is_reward_claimed(&ctx.reward_claimed, point_id, id) {
                continue;
            }

            set_reward_claimed(&mut ctx.reward_claimed, point_id, id, 1);
            ctx.reward_queue.push(vec![0x11, point_id, id]);

            match kind {
                1 if event_unit_slot(&ctx.event_unit_rows, target)? != -1 => {
                    let slot = event_unit_slot(&ctx.event_unit_rows, target)?;

                    ctx.set_i32_at(AppContext::EVENT_UNIT_OWNED.wrapping_add((slot as i64 * 4) as usize), 1)?;
                }
                0 => add_resource(ctx, target, amount, 0)?,
                _ => {}
            }

            point_reward_analytics(ctx, point_id, id, (cap <= total) as i32)?;
        }
    }

    ctx.set_block_at::<1>(AppContext::EX_OFFERED, [(ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63) as u8])?;

    let width = get_drawable_width(ctx)?;

    ctx.set_i32_at(AppContext::RESULT_OK_RECT, operation::div_2(width).wrapping_sub(0xbe))?;

    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
    let inset = get_bottom_inset_logical(ctx)?;

    ctx.set_i32_at(AppContext::RESULT_OK_RECT + 4, shift.wrapping_sub(inset).wrapping_add(0x226))?;
    ctx.set_i32_at(AppContext::RESULT_OK_RECT + 8, 0x17d)?;
    ctx.set_i32_at(AppContext::RESULT_OK_RECT + 0xc, 0x58)?;
    web_popup_request(ctx, 4);
    check_medals(ctx, 3)?;
    ad_prepare(ctx, 1)?;
    breadcrumb(ctx, 0x4a)
}
