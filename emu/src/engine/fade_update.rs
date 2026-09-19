use crate::Fault;

use super::{
    AppContext, ENTITY_BASE, ENTITY_STRIDE, Entity, FACTION_STRIDE, FormatArg, analytics_params,
    app_on_draw, bgm_player_switch, button_bank_remove, calculate_treasure_percentages,
    clear_cannon_shot, clear_crit_vfx_slot, clear_debris, clear_effect_slot, clear_items_selected,
    clear_wave_sprite, fever_clear_state, get_battle_status, get_entity_base_idx, get_entity_state,
    get_global_map_id, get_item_selected, get_max_hp, get_max_money, get_powerup_available,
    get_scene_id, get_stage_index, get_star_level, load_map_stage_csv, lose_exit_map_check,
    map_type_base_id, notification_schedule, record_stage_played, request_save_data,
    reset_hud_corner_rects, scene_transition_tick, set_battle_status, set_cannon_countdown,
    set_deck_cooldown, set_entity_state, set_hp, set_item_selected, set_money,
    set_powerup_available, set_scene, set_worker_level, sound_manager, validate_map_type,
    vibration_clear,
};

const SITE: &str = "fade_update";
const WALLET: usize = 0x2648;

enum Step {
    Advance,
    TreasureCalc,
    Dispatch,
    Tail,
    Stop,
    SaveStop,
    ResumeRevive,
}

pub fn fade_update(ctx: &mut AppContext, style: i32) -> Result<bool, Fault> {
    if ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0 {
        ctx.set_i32_at(AppContext::FADE_FRAME, 0)?;

        return Ok(true);
    }

    if style == 0 {
        let frame = ctx.i32_at(AppContext::FADE_FRAME)?;
        let started = ctx.u8_at(AppContext::FADE_STARTED)?;

        if frame == 0 {
            ctx.set_block_at::<1>(AppContext::FADE_STARTED, [1])?;
            ctx.set_i32_at(AppContext::FADE_FRAME, 1)?;
            calculate_treasure_percentages(ctx)?;
        } else {
            if frame == 0xb
                && started != 0
                && get_scene_id(ctx)? == 0x62
                && ctx
                    .scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .fade_menu_prompt()
            {
                app_on_draw(ctx)?;

                return Ok(false);
            }

            if frame == 0xa && started != 0 && get_scene_id(ctx)? == 0x62 {
                sound_manager(ctx)?.stop_audio(-1);
            }

            let current = ctx.i32_at(AppContext::FADE_FRAME)?;

            ctx.set_i32_at(AppContext::FADE_FRAME, current.wrapping_add(1))?;

            if current == 0 {
                calculate_treasure_percentages(ctx)?;
            }
        }

        if ctx.i32_at(AppContext::FADE_FRAME)? == 0xd {
            app_on_draw(ctx)?;

            let scene = get_scene_id(ctx)?;

            if scene == 0x64 || scene == 0x62 {
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .fade_menu_dispatch(0, scene);
            }
        }

        if ctx.i32_at(AppContext::FADE_FRAME)? < 0xd {
            return Ok(true);
        }

        ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [0])?;

        if ctx.i32_at(AppContext::SCENE_0X64_PAGE)? != 5 {
            request_save_data(ctx)?;
        }

        return Ok(false);
    }

    if style != 1 {
        return Ok(true);
    }

    let frame = ctx.i32_at(AppContext::FADE_FRAME)?;
    let mut step = Step::Advance;

    if frame == 0 {
        ctx.set_block_at::<1>(AppContext::FADE_STARTED, [1])?;
        ctx.set_i32_at(AppContext::FADE_FRAME, 1)?;
        step = Step::TreasureCalc;
    } else if frame == 0xa
        && ctx.u8_at(AppContext::FADE_STARTED)? != 0
        && get_scene_id(ctx)? == 0x12c
    {
        let status = get_battle_status(ctx)?;
        let collab = status == 1
            && ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
            && ctx.i32_at(AppContext::STAR_LEVEL)? == 0
            && {
                let map = map_type_base_id(
                    validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?),
                    ctx.i32_at(AppContext::MAP_INDEX)?,
                );
                let stage = ctx.i32_at(AppContext::STAGE_ROW)?;

                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .collab_reward_ready(map, stage)
            };

        if collab {
            if ctx.dialogs.active.is_empty() {
                let map = map_type_base_id(
                    validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?),
                    ctx.i32_at(AppContext::MAP_INDEX)?,
                );
                let stage = ctx.i32_at(AppContext::STAGE_ROW)?;

                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .collab_reward_dialog(map, stage);
            }

            step = Step::Stop;
        } else {
            let blocked = get_battle_status(ctx)? != 0
                && (get_battle_status(ctx)? != 1 || ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? != 0)
                && (get_battle_status(ctx)? != 2 || ctx.i32_at(AppContext::REVIVE_REQUESTED)? != 0);

            if !blocked
                && ctx
                    .scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .unlock_popup_pending()
            {
                step = Step::Stop;
            }
        }
    }

    loop {
        match step {
            Step::Advance => {
                let frame = ctx.i32_at(AppContext::FADE_FRAME)?;

                if frame == 0xd
                    && ctx
                        .scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .loader_busy()
                {
                    step = Step::Stop;

                    continue;
                }

                let frame = ctx.i32_at(AppContext::FADE_FRAME)?;

                ctx.set_i32_at(AppContext::FADE_FRAME, frame.wrapping_add(1))?;

                step = match frame {
                    0 => Step::TreasureCalc,
                    _ if frame.wrapping_add(1) == 0xc => Step::Dispatch,
                    _ => Step::Tail,
                };
            }
            Step::TreasureCalc => {
                calculate_treasure_percentages(ctx)?;
                step = if ctx.i32_at(AppContext::FADE_FRAME)? != 0xc {
                    Step::Tail
                } else {
                    Step::Dispatch
                };
            }
            Step::Tail => {
                if ctx.i32_at(AppContext::FADE_FRAME)? < 0x18 {
                    return Ok(true);
                }

                ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [0])?;
                step = Step::Stop;
            }
            Step::Stop => {
                app_on_draw(ctx)?;

                return Ok(false);
            }
            Step::SaveStop => {
                request_save_data(ctx)?;

                return Ok(false);
            }
            Step::ResumeRevive => {
                ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 0)?;
                ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30])?;
                ctx.set_block_at::<0x20>(AppContext::SCROLL_STATE, [0; 0x20])?;

                for slot in 0..0x33 {
                    if get_entity_state(ctx, 1, slot)? == 2 {
                        set_entity_state(ctx, 1, slot, 0)?;
                    }
                }

                reset_hud_corner_rects(ctx)?;
                ctx.set_block_at::<8>(AppContext::DECK_BAR_SLIDE, [0; 8])?;

                for button in 0..10 {
                    set_deck_cooldown(ctx, WALLET, button, 0, 1)?;
                }

                for offset in (0..0x380usize).step_by(0x10) {
                    clear_debris(ctx, AppContext::CAT_DEBRIS + offset)?;
                }

                for offset in (0..0x380usize).step_by(0x10) {
                    clear_debris(ctx, AppContext::ENEMY_DEBRIS + offset)?;
                }

                for slot in 0..0x1eusize {
                    clear_cannon_shot(ctx, AppContext::CANNON_SHOTS + slot * 0xc)?;
                }

                for offset in (0..0xc80usize).step_by(0x10) {
                    clear_crit_vfx_slot(ctx, AppContext::CRIT_VFX + offset)?;
                }

                let camera = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0x2580);

                ctx.set_i32_at(AppContext::CAMERA_X, camera)?;
                ctx.set_i32_at(AppContext::OUTRO_PHASE, 0)?;
                set_battle_status(ctx, 3)?;
                ctx.set_i32_at(ENTITY_BASE + Entity::STATE, 0)?;

                let hp = get_max_hp(ctx, 0, 0)?;

                set_hp(ctx, 0, 0, hp)?;

                let base = get_entity_base_idx(ctx)?;

                for slot in 1..0x31 {
                    if slot != base {
                        ctx.set_i32_at(
                            ENTITY_BASE
                                + FACTION_STRIDE
                                + slot as usize * ENTITY_STRIDE
                                + Entity::POS_X,
                            0xaf0,
                        )?;
                    }
                }

                if base != 0x31 {
                    ctx.set_i32_at(
                        ENTITY_BASE + FACTION_STRIDE + 0x31 * ENTITY_STRIDE + Entity::POS_X,
                        0xaf0,
                    )?;
                }

                if base != 0x32 {
                    ctx.set_i32_at(
                        ENTITY_BASE + FACTION_STRIDE + 0x32 * ENTITY_STRIDE + Entity::POS_X,
                        0xaf0,
                    )?;
                }

                bgm_player_switch(ctx, 0, 1)?;
                ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;
                ctx.set_block_at::<0x2c>(AppContext::SPEED_UP_PRESS, [0; 0x2c])?;
                ctx.set_block_at::<0x28>(AppContext::CAT_GOD_BUTTON_PRESS, [0; 0x28])?;
                ctx.set_block_at::<0x34>(AppContext::HUD_STATE, [0; 0x34])?;
                ctx.set_block_at::<0x2ba>(AppContext::CAMERA_KICK, [0; 0x2ba])?;
                ctx.set_block_at::<0x195>(AppContext::PENDING_STRIKE_SPEED, [0; 0x195])?;
                ctx.set_block_at::<0x508>(AppContext::SNIPER_FIRE_FRAME, [0; 0x508])?;
                ctx.set_block_at::<0x7c>(AppContext::OUTRO_OK_PRESS, [0; 0x7c])?;

                for offset in (0..0x2580usize).step_by(0x30) {
                    clear_effect_slot(ctx, AppContext::EFFECT_SLOTS + offset)?;
                }

                for offset in (0..0x2580usize).step_by(0x30) {
                    for pointer in 0..6usize {
                        clear_wave_sprite(ctx, AppContext::WAVE_SPRITES + offset + pointer * 8)?;
                    }
                }

                ctx.set_block_at::<0x27d8>(AppContext::WAVE_HITS, [0; 0x27d8])?;
                ctx.set_block_at::<0x30>(AppContext::PROC_ROLLS, [0; 0x30])?;
                ctx.set_block_at::<1>(AppContext::SNIPER_CASINGS_LIVE, [0])?;
                set_worker_level(ctx, WALLET, 7)?;

                let money = get_max_money(ctx, WALLET)?;

                set_money(ctx, WALLET, money)?;
                set_cannon_countdown(ctx, 0, 0)?;
                step = Step::SaveStop;
            }
            Step::Dispatch => {
                let scene = get_scene_id(ctx)?;

                if scene == 0x3e7 {
                    ctx.set_i32_at(AppContext::SCENE_0X64_PAGE, 9)?;
                    ctx.set_i32_at(AppContext::SCENE_0X64_PAGE_NEXT, -1)?;
                    set_scene(ctx, 0x12c)?;

                    step = if get_battle_status(ctx)? == 2
                        && ctx.i32_at(AppContext::REVIVE_REQUESTED)? == 1
                    {
                        Step::ResumeRevive
                    } else {
                        Step::SaveStop
                    };

                    continue;
                }

                if scene != 0x12c {
                    if scene == 0x63 || scene == 0x64 {
                        if ctx
                            .scene_host()
                            .ok_or(Fault::HostMissing { site: SITE })?
                            .fade_menu_dispatch(1, scene)
                        {
                            step = Step::Tail;

                            continue;
                        }

                        return Ok(false);
                    }

                    step = Step::Tail;

                    continue;
                }

                if get_battle_status(ctx)? | ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0 {
                    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                    if chapter == 0x63 {
                        ctx.set_i32_at(
                            AppContext::CHAPTER_MODE,
                            ctx.i32_at(AppContext::OUTRO_CHAPTER_MODE)?,
                        )?;
                        ctx.set_i32_at(
                            AppContext::ENTRY_STAGE,
                            ctx.i32_at(AppContext::OUTRO_ENTRY_STAGE)?,
                        )?;
                    }

                    set_scene(ctx, 0x64)?;
                    scene_transition_tick(ctx)?;
                    ctx.set_i32_at(AppContext::SCENE_0X64_PAGE, 0)?;
                    ctx.set_i32_at(AppContext::SCENE_0X64_PAGE_NEXT, -1)?;
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .battle_exit_cleanup((chapter == 0x63) as u8);
                    ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                    ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                    clear_items_selected(ctx)?;
                    request_save_data(ctx)?;
                    step = Step::Stop;

                    continue;
                }

                let status = get_battle_status(ctx)?;

                if status == 1 {
                    ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                    ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;

                    if ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? != 0 {
                        clear_items_selected(ctx)?;
                        ctx.scene_host()
                            .ok_or(Fault::HostMissing { site: SITE })?
                            .map_ui_reset();
                        set_scene(ctx, 0x63)?;
                        scene_transition_tick(ctx)?;
                        button_bank_remove(&mut ctx.buttons, 0xc8);
                        button_bank_remove(&mut ctx.buttons, 0xc9);
                        step = Step::Stop;

                        continue;
                    }

                    if ctx.u8_at(AppContext::CAT_FOOD_SHOP_OPEN)? != 0 {
                        step = Step::Stop;

                        continue;
                    }
                } else if status == 2 || get_battle_status(ctx)? == 0 {
                    if ctx.i32_at(AppContext::REVIVE_REQUESTED)? != 0
                        && get_battle_status(ctx)? != 0
                    {
                        if ctx.i32_at(AppContext::REVIVE_REQUESTED)? != 1 {
                            step = Step::Tail;

                            continue;
                        }

                        ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 0)?;
                        ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30])?;
                        ctx.set_block_at::<0x20>(AppContext::SCROLL_STATE, [0; 0x20])?;

                        for slot in 0..0x33 {
                            if get_entity_state(ctx, 1, slot)? == 2 {
                                set_entity_state(ctx, 1, slot, 0)?;
                            }
                        }

                        reset_hud_corner_rects(ctx)?;
                        ctx.set_block_at::<8>(AppContext::DECK_BAR_SLIDE, [0; 8])?;

                        for button in 0..10usize {
                            set_deck_cooldown(ctx, WALLET, button as i32, 0, 1)?;
                            ctx.set_i32_at(AppContext::DECK_COOLDOWN_VFX + button * 4, 0)?;
                        }

                        for offset in (0..0x380usize).step_by(0x10) {
                            clear_debris(ctx, AppContext::CAT_DEBRIS + offset)?;
                        }

                        for offset in (0..0x380usize).step_by(0x10) {
                            clear_debris(ctx, AppContext::ENEMY_DEBRIS + offset)?;
                        }

                        for slot in 0..0x1eusize {
                            clear_cannon_shot(ctx, AppContext::CANNON_SHOTS + slot * 0xc)?;
                        }

                        for offset in (0..0xc80usize).step_by(0x10) {
                            clear_crit_vfx_slot(ctx, AppContext::CRIT_VFX + offset)?;
                        }

                        let camera = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0x2580);

                        ctx.set_i32_at(AppContext::CAMERA_X, camera)?;
                        ctx.set_i32_at(AppContext::OUTRO_PHASE, 0)?;
                        set_battle_status(ctx, 3)?;
                        ctx.set_i32_at(ENTITY_BASE + Entity::STATE, 0)?;

                        let hp = get_max_hp(ctx, 0, 0)?;

                        set_hp(ctx, 0, 0, hp)?;

                        let base = get_entity_base_idx(ctx)?;

                        for slot in 1..0x31 {
                            if slot != base {
                                ctx.set_i32_at(
                                    ENTITY_BASE
                                        + FACTION_STRIDE
                                        + slot as usize * ENTITY_STRIDE
                                        + Entity::POS_X,
                                    0xaf0,
                                )?;
                            }
                        }

                        if base != 0x31 {
                            ctx.set_i32_at(
                                ENTITY_BASE + FACTION_STRIDE + 0x31 * ENTITY_STRIDE + Entity::POS_X,
                                0xaf0,
                            )?;
                        }

                        if base != 0x32 {
                            ctx.set_i32_at(
                                ENTITY_BASE + FACTION_STRIDE + 0x32 * ENTITY_STRIDE + Entity::POS_X,
                                0xaf0,
                            )?;
                        }

                        bgm_player_switch(ctx, 0, 1)?;
                        ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                        ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;
                        ctx.set_block_at::<0x2c>(AppContext::SPEED_UP_PRESS, [0; 0x2c])?;
                        ctx.set_block_at::<0x28>(AppContext::CAT_GOD_BUTTON_PRESS, [0; 0x28])?;
                        ctx.set_block_at::<0x34>(AppContext::HUD_STATE, [0; 0x34])?;
                        ctx.set_block_at::<0x2ba>(AppContext::CAMERA_KICK, [0; 0x2ba])?;
                        ctx.set_block_at::<0x195>(AppContext::PENDING_STRIKE_SPEED, [0; 0x195])?;
                        ctx.set_block_at::<0x509>(AppContext::SNIPER_FIRE_FRAME, [0; 0x509])?;
                        ctx.set_block_at::<0x7c>(AppContext::OUTRO_OK_PRESS, [0; 0x7c])?;

                        for offset in (0..0x2580usize).step_by(0x30) {
                            clear_effect_slot(ctx, AppContext::EFFECT_SLOTS + offset)?;
                        }

                        for offset in (0..0x2580usize).step_by(0x30) {
                            for pointer in 0..6usize {
                                clear_wave_sprite(
                                    ctx,
                                    AppContext::WAVE_SPRITES + offset + pointer * 8,
                                )?;
                            }
                        }

                        ctx.set_block_at::<0x27d8>(AppContext::WAVE_HITS, [0; 0x27d8])?;
                        ctx.surge_events.clear();
                        ctx.counter_surge_events.clear();
                        ctx.explosion_events.clear();
                        ctx.set_block_at::<0x30>(AppContext::PROC_ROLLS, [0; 0x30])?;
                        set_worker_level(ctx, WALLET, 7)?;

                        let money = get_max_money(ctx, WALLET)?;

                        set_money(ctx, WALLET, money)?;
                        set_cannon_countdown(ctx, 0, 0)?;
                        ctx.set_block_at::<1>(AppContext::OUTRO_VIDEO_WATCHED, [0])?;
                        ctx.set_i32_at(AppContext::DEPLOY_LIMIT_TOTAL, 0)?;
                        fever_clear_state(&mut ctx.special_rules);
                        vibration_clear(ctx);
                        step = Step::Tail;

                        continue;
                    }

                    if get_battle_status(ctx)? == 0
                        && ctx.i32_at(AppContext::BATTLE_INTRO_FRAME)? >= 0x14a
                    {
                        record_stage_played(ctx)?;
                    }

                    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

                    if chapter == 0x63 {
                        ctx.set_block_at::<1>(AppContext::EX_OFFERED, [0])?;
                        ctx.set_i32_at(
                            AppContext::CHAPTER_MODE,
                            ctx.i32_at(AppContext::OUTRO_CHAPTER_MODE)?,
                        )?;
                        ctx.set_i32_at(
                            AppContext::ENTRY_STAGE,
                            ctx.i32_at(AppContext::OUTRO_ENTRY_STAGE)?,
                        )?;
                    }

                    if get_battle_status(ctx)? == 0 {
                        ctx.scene_host()
                            .ok_or(Fault::HostMissing { site: SITE })?
                            .battle_exit_cleanup((chapter == 0x63) as u8);
                        ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                        ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                    } else if get_battle_status(ctx)? == 2
                        && ctx.u8_at(AppContext::LEADERSHIP_REFUND)? != 0
                    {
                        ctx.scene_host()
                            .ok_or(Fault::HostMissing { site: SITE })?
                            .leadership_refund();

                        let map_id = get_global_map_id(ctx, 0)?;
                        let stage = get_stage_index(ctx)?;
                        let star = get_star_level(ctx)?;
                        let total = ctx.i32_at(AppContext::LEADERSHIP_TOTAL)?;

                        analytics_params(
                            ctx,
                            0x98e88d,
                            total,
                            &[
                                (b"sec1_type", FormatArg::Text(b"MapID")),
                                (b"sec1_id", FormatArg::Int(map_id)),
                                (b"sec2_type", FormatArg::Text(b"StageIdx")),
                                (b"sec2_id", FormatArg::Int(stage)),
                                (b"ex_type", FormatArg::Text(b"StageLv")),
                                (b"ex_id", FormatArg::Int(star)),
                            ],
                        )?;

                        let notice = ctx.u8_at(AppContext::LEADERSHIP_NOTICE)?;

                        notification_schedule(ctx, notice, 1)?;
                    }

                    ctx.set_block_at::<0x2c>(AppContext::SPEED_UP_PRESS, [0; 0x2c])?;
                    ctx.set_block_at::<0x28>(AppContext::CAT_GOD_BUTTON_PRESS, [0; 0x28])?;
                    ctx.set_block_at::<0x34>(AppContext::HUD_STATE, [0; 0x34])?;
                    ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;
                    ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                    ctx.set_block_at::<0x7c>(AppContext::OUTRO_OK_PRESS, [0; 0x7c])?;

                    let tutorial = ctx.i32_at(AppContext::TUTORIAL_CLEARED)?;

                    clear_items_selected(ctx)?;
                    set_scene(ctx, 0x64)?;
                    scene_transition_tick(ctx)?;

                    if tutorial == 0 {
                        ctx.set_i32_at(AppContext::SCENE_0X64_PAGE, 0)?;
                        ctx.set_i32_at(AppContext::SCENE_0X64_PAGE_NEXT, -1)?;

                        return Ok(false);
                    }

                    let direct = ctx.u8_at(AppContext::OUTRO_EXIT_DIRECT)? as i32;

                    ctx.set_i32_at(
                        AppContext::SCENE_0X64_PAGE,
                        direct.wrapping_mul(4).wrapping_add(5),
                    )?;
                    ctx.set_i32_at(AppContext::SCENE_0X64_PAGE_NEXT, -1)?;
                    button_bank_remove(&mut ctx.buttons, 0xc8);
                    button_bank_remove(&mut ctx.buttons, 0xc9);

                    if ctx.u8_at(AppContext::OUTRO_EXIT_DIRECT)? != 0 {
                        let entry = ctx.i32_at(AppContext::LOSE_ENTRY_CHAPTER)?;

                        if (entry == 0x62 || entry == 3) && !lose_exit_map_check(ctx)? {
                            ctx.set_i32_at(AppContext::MAP_RETURN_FLAG, 1)?;
                        }
                    }

                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_return_reset();
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_return_flags();
                    ctx.set_block_at::<8>(AppContext::DECK_ROW_SWAPPING + 6, [0; 8])?;
                    ctx.set_block_at::<8>(AppContext::DECK_ROW_SWAPPING, [0; 8])?;
                    ctx.set_block_at::<0x3c>(AppContext::SETUP_FRAMES - 4, [0; 0x3c])?;

                    for text in ctx.menu_texts.iter_mut() {
                        *text = None;
                    }

                    ctx.set_block_at::<2>(AppContext::MENU_TEXTURE_PAGE, [0; 2])?;
                    ctx.set_block_at::<8>(
                        AppContext::MENU_TEXTURE_PAGE + 0x30,
                        0x1ca00000000u64.to_le_bytes(),
                    )?;
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_screen_init(0);
                    ctx.set_block_at::<0x10>(AppContext::MENU_CURSOR, [0xff; 0x10])?;
                    ctx.set_block_at::<8>(AppContext::MENU_CURSOR + 0x10, [0xff; 8])?;
                    ctx.set_i32_at(AppContext::MENU_BUILD_MODE, 2)?;
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_menu_build(1);
                    ctx.set_i32_at(AppContext::MENU_BUILD_MODE, 0)?;
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_background_pick();
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .scene_background_setup();
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_ui_reset();
                    sound_manager(ctx)?.stop_audio(-1);
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_bgm_start(0);
                    step = Step::Stop;

                    continue;
                } else if get_battle_status(ctx)? == 4 {
                    ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                    ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;

                    if ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? != 0 {
                        clear_items_selected(ctx)?;
                        ctx.scene_host()
                            .ok_or(Fault::HostMissing { site: SITE })?
                            .map_ui_reset();
                        set_scene(ctx, 0x63)?;
                        scene_transition_tick(ctx)?;
                        button_bank_remove(&mut ctx.buttons, 0xc8);
                        button_bank_remove(&mut ctx.buttons, 0xc9);
                        step = Step::Stop;

                        continue;
                    }

                    if ctx.u8_at(AppContext::CAT_FOOD_SHOP_OPEN)? != 0 {
                        return Ok(false);
                    }
                } else if get_battle_status(ctx)? == 7 {
                    ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                    ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;

                    if ctx.i32_at(AppContext::OUTRO_MAP_LOCKED)? != 0
                        || ctx.u8_at(AppContext::CAT_FOOD_SHOP_OPEN)? != 0
                    {
                        return Ok(false);
                    }
                } else {
                    clear_items_selected(ctx)?;
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_ui_reset();
                    set_scene(ctx, 0x63)?;
                    scene_transition_tick(ctx)?;
                    button_bank_remove(&mut ctx.buttons, 0xc8);
                    button_bank_remove(&mut ctx.buttons, 0xc9);
                    step = Step::Stop;

                    continue;
                }

                let item = get_item_selected(ctx, 0)?;
                let powerup = get_powerup_available(ctx)?;

                clear_items_selected(ctx)?;
                button_bank_remove(&mut ctx.buttons, 0xc8);
                button_bank_remove(&mut ctx.buttons, 0xc9);

                if ctx.u8_at(AppContext::EX_ACCEPTED)? != 0 {
                    set_item_selected(ctx, 0, item as u8)?;
                    set_powerup_available(ctx, powerup)?;
                    ctx.set_block_at::<1>(AppContext::EX_ACCEPTED, [0])?;
                    ctx.set_i32_at(AppContext::ENTRY_STAGE, ctx.i32_at(AppContext::EX_STAGE)?)?;
                    ctx.set_i32_at(AppContext::CHAPTER_MODE, 0x63)?;
                    ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
                    ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;
                    app_on_draw(ctx)?;

                    let ex_map = ctx.i32_at(AppContext::EX_MAP)?;

                    if load_map_stage_csv(ctx, ex_map, 0, 1, 0, 1, 1)? {
                        request_save_data(ctx)?;

                        if status == 1 {
                            ctx.scene_host()
                                .ok_or(Fault::HostMissing { site: SITE })?
                                .leadership_return_begin();
                        }

                        set_scene(ctx, 0x12c)?;
                    }

                    return Ok(false);
                }

                if ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63 {
                    ctx.set_i32_at(
                        AppContext::CHAPTER_MODE,
                        ctx.i32_at(AppContext::OUTRO_CHAPTER_MODE)?,
                    )?;
                    ctx.set_i32_at(
                        AppContext::ENTRY_STAGE,
                        ctx.i32_at(AppContext::OUTRO_ENTRY_STAGE)?,
                    )?;
                }

                if status == 1 {
                    ctx.scene_host()
                        .ok_or(Fault::HostMissing { site: SITE })?
                        .map_return_reset();
                }

                set_scene(ctx, 0x64)?;
                scene_transition_tick(ctx)?;
                ctx.set_block_at::<8>(AppContext::DECK_ROW_SWAPPING + 6, [0; 8])?;
                ctx.set_block_at::<8>(AppContext::DECK_ROW_SWAPPING, [0; 8])?;
                ctx.set_block_at::<0x3c>(AppContext::SETUP_FRAMES - 4, [0; 0x3c])?;

                let direct = ctx.u8_at(AppContext::OUTRO_EXIT_DIRECT)? as i32;

                ctx.set_i32_at(
                    AppContext::SCENE_0X64_PAGE,
                    direct.wrapping_mul(4).wrapping_add(5),
                )?;
                ctx.set_i32_at(AppContext::SCENE_0X64_PAGE_NEXT, -1)?;

                if direct != 0 {
                    let entry = ctx.i32_at(AppContext::LOSE_ENTRY_CHAPTER)?;

                    if (entry == 0x62 || entry == 3) && (status != 1 || !lose_exit_map_check(ctx)?)
                    {
                        ctx.set_i32_at(AppContext::MAP_RETURN_FLAG, 1)?;
                    }
                }

                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_return_flags();
                ctx.set_block_at::<0x2c>(AppContext::SPEED_UP_PRESS, [0; 0x2c])?;
                ctx.set_block_at::<0x28>(AppContext::CAT_GOD_BUTTON_PRESS, [0; 0x28])?;
                ctx.set_block_at::<0x34>(AppContext::HUD_STATE, [0; 0x34])?;
                ctx.set_i32_at(AppContext::SWIPE_STATE, 0)?;
                ctx.set_block_at::<8>(AppContext::SWIPE_VELOCITY, [0; 8])?;
                ctx.set_block_at::<2>(AppContext::MENU_TEXTURE_PAGE, [0; 2])?;
                ctx.set_block_at::<8>(
                    AppContext::MENU_TEXTURE_PAGE + 0x30,
                    0x1ca00000000u64.to_le_bytes(),
                )?;
                ctx.set_block_at::<0x7c>(AppContext::OUTRO_OK_PRESS, [0; 0x7c])?;
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_screen_init(0);
                ctx.set_block_at::<0x10>(AppContext::MENU_CURSOR, [0xff; 0x10])?;
                ctx.set_block_at::<8>(AppContext::MENU_CURSOR + 0x10, [0xff; 8])?;
                ctx.set_i32_at(AppContext::MENU_BUILD_MODE, 2)?;
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_menu_build(1);
                ctx.set_i32_at(AppContext::MENU_BUILD_MODE, 0)?;
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_background_pick();
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .scene_background_setup();
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_ui_reset();
                sound_manager(ctx)?.stop_audio(-1);
                ctx.scene_host()
                    .ok_or(Fault::HostMissing { site: SITE })?
                    .map_bgm_start(0);
                ctx.set_i32_at(AppContext::TUTORIAL_CLEARED, 1)?;
                ctx.set_i32_at(AppContext::CAT_FOOD_SHOP_ENABLED, 1)?;

                if ctx.i32_at(AppContext::SCENE_0X64_PAGE)? != 5 {
                    request_save_data(ctx)?;
                }

                if status == 1 {
                    app_on_draw(ctx)?;
                }

                return Ok(false);
            }
        }
    }
}
