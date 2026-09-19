use std::{cell, collections::BTreeMap, rc::Rc};

use crate::Fault;

use super::{
    AppContext, Imgcut, Maanim, Mamodel, deck_slot_filled, format_localized, get_altar_level_cap,
    get_background_id, get_bg_image_id, get_bg_model_id, get_button_unit_form, get_button_unit_id,
    get_castle_enemy_row, get_map_type, get_text_texture, imgcut_get_width, is_scored_stage,
    is_space_map, maanim_initialize, maanim_load, mamodel_get_part, mamodel_load,
    mamodel_set_sheet, mamodel_set_sheet_table, mamodel_set_single_sheet, map_type_code,
    query_localizable, stat_soul_animation_type, stat_spawn_animation_type,
    std_map_int_maanim_subscript, string_format_int, string_format_int2,
    string_format_int2_text_copy, string_format_int2_text2, string_format_int3,
    string_format_rank_comment, string_split, text_texture_cache, texture_cache_load,
    texture_context_init, validate_map_type,
};

const SITE: &str = "load_battle_assets";

#[allow(clippy::if_same_then_else)]
pub fn load_battle_assets(ctx: &mut AppContext) -> Result<(), Fault> {
    texture_context_init(ctx)?;

    ctx.bg_sheet = None;
    ctx.img001_sheet = None;
    ctx.img024_sheet = None;
    ctx.img060_sheet = None;
    ctx.mapicon_sheet = None;
    ctx.img002_sheet = None;
    ctx.stage_name_sheet = None;
    ctx.img003_sheet = None;
    ctx.img004_sheet = None;
    ctx.img043_sheet = None;
    ctx.img006_sheet = None;
    ctx.img040_sheet = None;
    ctx.img041_sheet = None;
    ctx.img042_sheet = None;
    ctx.img100_sheet = None;
    ctx.img101_sheet = None;
    ctx.img001_second_sheet = None;
    ctx.bubble_sheet = None;
    ctx.outro_event_sheets = [None, None, None];
    ctx.castle_sheet = None;
    ctx.castle_models = Default::default();
    ctx.castle_anims = Default::default();
    ctx.effect_a_sheet = None;
    ctx.boss_welcome_model = Mamodel::default();
    ctx.crit_vfx_model = Mamodel::default();
    ctx.boss_shockwave_anim = Maanim::default();
    ctx.crit_vfx_anim = Maanim::default();
    ctx.zombie_sheets = [None, None];
    ctx.zombie_model = Mamodel::default();
    ctx.zombie_down_anim = Maanim::default();
    ctx.zombie_revive_anim = Maanim::default();
    ctx.zombie_back_anim = Maanim::default();
    ctx.skill_sheets = (0..0x18).map(|_| cell::Cell::new(None)).collect();
    ctx.skill_up_model = Mamodel::default();
    ctx.skill_up_anim = Maanim::default();
    ctx.skill_slow_model = Mamodel::default();
    ctx.skill_slow_anim = Maanim::default();
    ctx.skill_stop_model = Mamodel::default();
    ctx.skill_stop_anim = Maanim::default();
    ctx.skill_shield_model = Mamodel::default();
    ctx.skill_shield_anim = Maanim::default();
    ctx.skill_down_model = Mamodel::default();
    ctx.skill_down_anim = Maanim::default();
    ctx.skill_wave_invalid_model = Mamodel::default();
    ctx.skill_wave_invalid_anim = Maanim::default();
    ctx.skill_wave_stop_model = Mamodel::default();
    ctx.skill_wave_stop_anim = Maanim::default();
    ctx.skill_up_e_model = Mamodel::default();
    ctx.skill_up_e_anim = Maanim::default();
    ctx.skill_slow_e_model = Mamodel::default();
    ctx.skill_slow_e_anim = Maanim::default();
    ctx.skill_stop_e_model = Mamodel::default();
    ctx.skill_stop_e_anim = Maanim::default();
    ctx.skill_shield_e_model = Mamodel::default();
    ctx.skill_shield_e_anim = Maanim::default();
    ctx.skill_down_e_model = Mamodel::default();
    ctx.skill_down_e_anim = Maanim::default();
    ctx.skill_wave_invalid_e_model = Mamodel::default();
    ctx.skill_wave_invalid_e_anim = Maanim::default();
    ctx.skill_wave_stop_e_model = Mamodel::default();
    ctx.skill_wave_stop_e_anim = Maanim::default();
    ctx.skill_effect_invalid_model = Mamodel::default();
    ctx.skill_effect_invalid_anim = Maanim::default();
    ctx.skill_zombie_strong_model = Mamodel::default();
    ctx.zkill_vfx_anim = Maanim::default();
    ctx.barrier_model = Mamodel::default();
    ctx.barrier_anims = Default::default();
    ctx.demonshield_model = Mamodel::default();
    ctx.shield_anims = Default::default();
    ctx.guard_e_model = Mamodel::default();
    ctx.guard_e_anim = Maanim::default();
    ctx.guard_e_breaker_anim = Maanim::default();
    ctx.demonsoul_01_model = Mamodel::default();
    ctx.demonsoul_00_model = Mamodel::default();
    ctx.death_surge_anims = Default::default();
    ctx.demonsoul_sheets = [None, None];
    ctx.demonsummon_model = Mamodel::default();
    ctx.demonsummon_e_model = Mamodel::default();
    ctx.counter_surge_anims = Default::default();
    ctx.warp_model = Mamodel::default();
    ctx.warp_chara_model = Mamodel::default();
    ctx.warp_anims = Default::default();
    ctx.skill_curse_model = Mamodel::default();
    ctx.skill_curse_e_model = Mamodel::default();
    ctx.skill_curse_anim = Maanim::default();
    ctx.skill_curse_e_anim = Maanim::default();
    ctx.wave_attack_model = Mamodel::default();
    ctx.wave_attack_e_model = Mamodel::default();
    ctx.smallwave_attack_model = Mamodel::default();
    ctx.smallwave_attack_e_model = Mamodel::default();
    ctx.wave_anim = Maanim::default();
    ctx.wave_attack_e_anim = Maanim::default();
    ctx.mini_wave_anim = Maanim::default();
    ctx.smallwave_attack_e_anim = Maanim::default();
    ctx.explosion_model = Mamodel::default();
    ctx.explosion_e_model = Mamodel::default();
    ctx.explosion_anims = Default::default();
    ctx.effect_sheets.clear();
    ctx.effect_models.clear();
    ctx.effect_anims.clear();
    ctx.strong_attack_model = Mamodel::default();
    ctx.attack_invalid_model = Mamodel::default();
    ctx.percentage_attack_model = Mamodel::default();
    ctx.savage_vfx_anim = Maanim::default();
    ctx.attack_invalid_anim = Maanim::default();
    ctx.toxic_vfx_anim = Maanim::default();
    ctx.volcano_model = Mamodel::default();
    ctx.volcano_e_model = Mamodel::default();
    ctx.smallvolcano_model = Mamodel::default();
    ctx.smallvolcano_e_model = Mamodel::default();
    ctx.volcano_anims = Default::default();
    ctx.smallvolcano_anims = Default::default();
    ctx.unit_sheets = [
        (0..0x36c).map(|_| cell::Cell::new(None)).collect(),
        (0..0x36c).map(|_| cell::Cell::new(None)).collect(),
        (0..0x36c).map(|_| cell::Cell::new(None)).collect(),
        (0..0x36c).map(|_| cell::Cell::new(None)).collect(),
    ];
    ctx.unit_models[0] = (0..0x15).map(|_| Mamodel::default()).collect();
    ctx.unit_anims[0] = (0..0x15).map(|_| BTreeMap::new()).collect();
    ctx.enemy_sheets = (0..0x322).map(|_| cell::Cell::new(None)).collect();
    ctx.unit_models[1] = (0..0xb).map(|_| Mamodel::default()).collect();
    ctx.unit_anims[1] = (0..0xb).map(|_| BTreeMap::new()).collect();
    ctx.bg_anim_cache.clear();
    ctx.img015_sheet = None;
    ctx.equipment_attribute_sheet = None;
    ctx.equipment_effect_sheet = None;
    ctx.equipment_grade_sheet = None;
    ctx.equipment_shadow_sheet = None;
    ctx.unit_info_texts = [None, None, None, None];
    ctx.equipment_attribute_s_sheet = None;
    ctx.equipment_effect_s_sheet = None;
    ctx.invoke_equipment_model = Mamodel::default();
    ctx.invoke_equipment_anim = Maanim::default();
    ctx.sealed_announce_sheets = [None, None, None];
    ctx.demonbattle_model = Mamodel::default();
    ctx.demon_banner_anim = Maanim::default();
    ctx.metal_strong_model = Mamodel::default();
    ctx.metal_killer_vfx_anim = Maanim::default();
    ctx.recast_decrease_e_model = Mamodel::default();
    ctx.drain_vfx_anim = Maanim::default();
    ctx.fever_sheet = None;
    ctx.fever_model = Mamodel::default();
    ctx.fever_anim = Maanim::default();

    let image = if get_bg_image_id(ctx)? != -1 {
        get_bg_image_id(ctx)?
    } else {
        ctx.i32_at(AppContext::STAGE_BACKGROUND_ID)?
    };
    let name = string_format_int(ctx, b"bg%03d.png", image)?;
    let png = query_localizable(ctx, &name);
    let model = get_bg_model_id(ctx)?;
    let name = string_format_int(ctx, b"bg%02d.imgcut", model)?;
    let cut = query_localizable(ctx, &name);

    ctx.bg_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    ctx.img001_sheet = texture_cache_load(ctx, b"img001.png", b"img001.imgcut", 0x2601)?;
    ctx.img024_sheet = texture_cache_load(ctx, b"img024.png", b"img024.imgcut", 0x2601)?;
    ctx.img060_sheet = texture_cache_load(ctx, b"img060_02.png", b"img060_02.imgcut", 0x2601)?;

    let png = query_localizable(ctx, b"mapicon.png");
    let cut = query_localizable(ctx, b"mapicon.imgcut");

    ctx.mapicon_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    ctx.img002_sheet = texture_cache_load(ctx, b"img002.png", b"img002.imgcut", 0x2601)?;

    let chapter_mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

    if chapter_mode == 3 {
        let code = map_type_code(validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?));
        let map = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name =
            string_format_int2_text2(ctx, b"mapsn%03d_%02d_%@_%@.png", map, stage, &code, &lang)?;
        let png = query_localizable(ctx, &name);
        let map = ctx.i32_at(AppContext::MAP_INDEX)?;
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_int2_text2(
            ctx,
            b"mapsn%03d_%02d_%@_%@.imgcut",
            map,
            stage,
            &code,
            &lang,
        )?;
        let cut = query_localizable(ctx, &name);

        ctx.stage_name_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    } else if chapter_mode == 0x63 {
        let map = ctx.i32_at(AppContext::EX_MAP_INDEX)?;
        let stage = ctx.i32_at(AppContext::EX_STAGE_INDEX)?;
        let lang = query_localizable(ctx, b"lang");
        let name =
            string_format_int2_text_copy(ctx, b"mapsn%03d_%02d_ex_%@.png", map, stage, &lang)?;
        let png = query_localizable(ctx, &name);
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_int2_text_copy(ctx, b"mapsnALL_all.imgcut", 0, 0, &lang)?;
        let cut = query_localizable(ctx, &name);

        ctx.stage_name_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    } else if get_map_type(ctx, 0)? == -2 || get_map_type(ctx, 0)? == -12 {
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"ec%03d_n_%@.png", stage, &lang)?;
        let png = query_localizable(ctx, &name);
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"mapsnALL_all.imgcut", stage, &lang)?;
        let cut = query_localizable(ctx, &name);

        ctx.stage_name_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    } else if get_map_type(ctx, 0)? == -3 || get_map_type(ctx, 0)? == -13 {
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"wc%03d_n_%@.png", stage, &lang)?;
        let png = query_localizable(ctx, &name);
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"mapsnALL_all.imgcut", stage, &lang)?;
        let cut = query_localizable(ctx, &name);

        ctx.stage_name_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    } else if is_space_map(ctx)? {
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"sc%03d_n_%@.png", stage, &lang)?;
        let png = query_localizable(ctx, &name);
        let stage = ctx.i32_at(AppContext::CASTLE_ID)?;
        let lang = query_localizable(ctx, b"lang");
        let name = string_format_rank_comment(ctx, b"mapsnALL_all.imgcut", stage, &lang)?;
        let cut = query_localizable(ctx, &name);

        ctx.stage_name_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
    }

    ctx.img003_sheet = texture_cache_load(ctx, b"img003.png", b"img003.imgcut", 0x2601)?;

    let scored = is_scored_stage(ctx)?;

    ctx.img004_sheet = if scored {
        let lang = query_localizable(ctx, b"lang");
        let png = format_localized(ctx, b"img004_DOJO_%@.png", &lang)?;
        let lang = query_localizable(ctx, b"lang");
        let cut = format_localized(ctx, b"img004_DOJO_%@.imgcut", &lang)?;

        texture_cache_load(ctx, &png, &cut, 0x2601)?
    } else {
        texture_cache_load(ctx, b"img004.png", b"img004.imgcut", 0x2601)?
    };
    ctx.img043_sheet = texture_cache_load(ctx, b"img043.png", b"img043.imgcut", 0x2601)?;
    ctx.img006_sheet = texture_cache_load(ctx, b"img006.png", b"img006.imgcut", 0x2601)?;
    ctx.img040_sheet = texture_cache_load(ctx, b"img040.png", b"img040.imgcut", 0x2601)?;
    ctx.img041_sheet = texture_cache_load(ctx, b"img041.png", b"img041.imgcut", 0x2601)?;
    ctx.img042_sheet = texture_cache_load(ctx, b"img042.png", b"img042.imgcut", 0x2601)?;
    ctx.img100_sheet = texture_cache_load(ctx, b"img100.png", b"img100.imgcut", 0x2601)?;

    if ctx.i32_at(AppContext::SCENE_0X64_PAGE)? == 5 && is_scored_stage(ctx)? {
        ctx.img101_sheet = texture_cache_load(
            ctx,
            b"img101_nekoDojo.png",
            b"img101_nekoDojo.imgcut",
            0x2601,
        )?;
    } else if ctx
        .img101_sheet
        .as_ref()
        .is_none_or(|sheet| sheet.png.as_slice() != b"img101_nekoDojo.png")
    {
        ctx.img101_sheet = texture_cache_load(ctx, b"img101.png", b"img101.imgcut", 0x2601)?;
    }

    ctx.img001_second_sheet = texture_cache_load(ctx, b"img001.png", b"img001.imgcut", 0x2601)?;

    if get_background_id(ctx)? == 0xd {
        let png = query_localizable(ctx, b"bubble.png");

        ctx.bubble_sheet = texture_cache_load(ctx, &png, b"", 0x2601)?;
    } else if get_background_id(ctx)? == 0xf || get_background_id(ctx)? == 0x48 {
        let png = query_localizable(ctx, b"bubble02.png");

        ctx.bubble_sheet = texture_cache_load(ctx, &png, b"", 0x2601)?;
    } else if get_background_id(ctx)? == 0x28 {
        let png = query_localizable(ctx, b"bubble03_bg040.png");

        ctx.bubble_sheet = texture_cache_load(ctx, &png, b"", 0x2601)?;
    } else if get_background_id(ctx)? == 0x2e || get_background_id(ctx)? == 0x2f {
        let png = query_localizable(ctx, b"bubble03_bg040.png");

        ctx.bubble_sheet = texture_cache_load(ctx, &png, b"", 0x2601)?;
    }

    let name = string_format_int(ctx, b"%03d_g.png", 0)?;
    let png = query_localizable(ctx, &name);
    let name = string_format_int(ctx, b"%03d_g.imgcut", 0)?;
    let cut = query_localizable(ctx, &name);

    ctx.castle_sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

    for (index, (part, variant)) in [(0, 1), (0, 2), (1, 1), (1, 2), (2, 1), (2, 2)]
        .into_iter()
        .enumerate()
    {
        let name = string_format_int3(ctx, b"%03d_g%02d_%d.mamodel", 0, part, variant)?;
        let path = query_localizable(ctx, &name);
        let mut model = std::mem::take(&mut ctx.castle_models[index]);

        mamodel_load(ctx, &mut model, &path)?;

        let name = string_format_int3(ctx, b"%03d_g%02d_%d.maanim", 0, part, variant)?;
        let path = query_localizable(ctx, &name);
        let mut anim = std::mem::take(&mut ctx.castle_anims[index]);

        maanim_load(ctx, &mut anim, &path)?;
        ctx.castle_anims[index] = anim;
        mamodel_set_sheet_table(
            &mut model,
            &Rc::from([cell::Cell::new(ctx.castle_sheet.clone())]),
        );
        maanim_initialize(&mut model, 0)?;
        ctx.castle_models[index] = model;
    }

    let name = string_format_int2(ctx, b"%03d_g%02d.mamodel", 0, 3)?;
    let path = query_localizable(ctx, &name);
    let mut model = std::mem::take(&mut ctx.castle_models[6]);

    mamodel_load(ctx, &mut model, &path)?;

    let name = string_format_int2(ctx, b"%03d_g%02d.maanim", 0, 3)?;
    let path = query_localizable(ctx, &name);
    let mut anim = std::mem::take(&mut ctx.castle_anims[6]);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.castle_anims[6] = anim;
    mamodel_set_sheet_table(
        &mut model,
        &Rc::from([cell::Cell::new(ctx.castle_sheet.clone())]),
    );
    maanim_initialize(&mut model, 0)?;
    ctx.castle_models[6] = model;
    ctx.effect_a_sheet = texture_cache_load(ctx, b"000_a.png", b"000_a.imgcut", 0x2601)?;

    let path = query_localizable(ctx, b"boss_welcome.mamodel");
    let mut boss_model = std::mem::take(&mut ctx.boss_welcome_model);

    mamodel_load(ctx, &mut boss_model, &path)?;

    let path = query_localizable(ctx, b"boss_welcome.maanim");
    let mut anim = std::mem::take(&mut ctx.boss_shockwave_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.boss_shockwave_anim = anim;

    let path = query_localizable(ctx, b"critical.mamodel");
    let mut crit_model = std::mem::take(&mut ctx.crit_vfx_model);

    mamodel_load(ctx, &mut crit_model, &path)?;

    let path = query_localizable(ctx, b"critical.maanim");
    let mut anim = std::mem::take(&mut ctx.crit_vfx_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.crit_vfx_anim = anim;
    mamodel_set_sheet_table(
        &mut boss_model,
        &Rc::from([cell::Cell::new(ctx.effect_a_sheet.clone())]),
    );
    maanim_initialize(&mut boss_model, 0)?;
    mamodel_set_sheet_table(
        &mut crit_model,
        &Rc::from([cell::Cell::new(ctx.effect_a_sheet.clone())]),
    );
    maanim_initialize(&mut crit_model, 0)?;
    ctx.boss_welcome_model = boss_model;
    ctx.crit_vfx_model = crit_model;
    ctx.zombie_sheets[1] = texture_cache_load(
        ctx,
        b"set_enemy001_zombie.png",
        b"set_enemy001_zombie.imgcut",
        0x2601,
    )?;

    let mut model = std::mem::take(&mut ctx.zombie_model);

    mamodel_load(ctx, &mut model, b"set_enemy001_zombie.mamodel")?;
    mamodel_set_sheet_table(
        &mut model,
        &Rc::from([
            cell::Cell::new(ctx.zombie_sheets[0].clone()),
            cell::Cell::new(ctx.zombie_sheets[1].clone()),
        ]),
    );
    ctx.zombie_model = model;

    let mut anim = std::mem::take(&mut ctx.zombie_down_anim);

    maanim_load(ctx, &mut anim, b"set_enemy001_zombie_down.maanim")?;
    ctx.zombie_down_anim = anim;

    let mut anim = std::mem::take(&mut ctx.zombie_revive_anim);

    maanim_load(ctx, &mut anim, b"set_enemy001_zombie_revive.maanim")?;
    ctx.zombie_revive_anim = anim;

    let mut anim = std::mem::take(&mut ctx.zombie_back_anim);

    maanim_load(ctx, &mut anim, b"set_enemy001_zombie_back.maanim")?;
    ctx.zombie_back_anim = anim;

    for skill in 0..0x18 {
        let name = string_format_int(ctx, b"skill%03d.png", skill)?;
        let png = query_localizable(ctx, &name);
        let name = string_format_int(ctx, b"skill%03d.imgcut", skill)?;
        let cut = query_localizable(ctx, &name);

        let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

        ctx.skill_sheets
            .get(skill as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: skill as i64,
                limit: ctx.skill_sheets.len() as i64,
            })?
            .set(sheet);
    }

    let path = query_localizable(ctx, b"skill_up.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_up_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_up.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_up_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_up_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_up_model = model;

    let path = query_localizable(ctx, b"skill_slow.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_slow_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_slow.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_slow_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_slow_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_slow_model = model;

    let path = query_localizable(ctx, b"skill_stop.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_stop_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_stop.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_stop_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_stop_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_stop_model = model;

    let path = query_localizable(ctx, b"skill_shield.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_shield_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_shield.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_shield_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_shield_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_shield_model = model;

    let path = query_localizable(ctx, b"skill_down.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_down_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_down.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_down_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_down_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_down_model = model;

    let path = query_localizable(ctx, b"skill_wave_invalid.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_wave_invalid_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_wave_invalid.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_wave_invalid_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_wave_invalid_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_wave_invalid_model = model;

    let path = query_localizable(ctx, b"skill_wave_stop.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_wave_stop_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_wave_stop.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_wave_stop_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_wave_stop_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_wave_stop_model = model;

    let path = query_localizable(ctx, b"skill_up_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_up_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_up_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_up_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_up_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_up_e_model = model;

    let path = query_localizable(ctx, b"skill_slow_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_slow_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_slow_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_slow_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_slow_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_slow_e_model = model;

    let path = query_localizable(ctx, b"skill_stop_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_stop_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_stop_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_stop_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_stop_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_stop_e_model = model;

    let path = query_localizable(ctx, b"skill_shield_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_shield_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_shield_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_shield_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_shield_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_shield_e_model = model;

    let path = query_localizable(ctx, b"skill_down_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_down_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_down_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_down_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_down_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_down_e_model = model;

    let path = query_localizable(ctx, b"skill_wave_invalid_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_wave_invalid_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_wave_invalid_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_wave_invalid_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_wave_invalid_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_wave_invalid_e_model = model;

    let path = query_localizable(ctx, b"skill_wave_stop_e.mamodel");
    let mut model = std::mem::take(&mut ctx.skill_wave_stop_e_model);

    mamodel_load(ctx, &mut model, &path)?;

    let path = query_localizable(ctx, b"skill_wave_stop_e.maanim");
    let mut anim = std::mem::take(&mut ctx.skill_wave_stop_e_anim);

    maanim_load(ctx, &mut anim, &path)?;
    ctx.skill_wave_stop_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_wave_stop_e_model = model;

    let mut model = std::mem::take(&mut ctx.skill_effect_invalid_model);

    mamodel_load(ctx, &mut model, b"skill_effect_invalid.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.skill_effect_invalid_anim);

    maanim_load(ctx, &mut anim, b"skill_effect_invalid.maanim")?;
    ctx.skill_effect_invalid_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_effect_invalid_model = model;

    let mut model = std::mem::take(&mut ctx.skill_zombie_strong_model);

    mamodel_load(ctx, &mut model, b"skill_zombie_strong.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.zkill_vfx_anim);

    maanim_load(ctx, &mut anim, b"skill_zombie_strong.maanim")?;
    ctx.zkill_vfx_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_zombie_strong_model = model;

    let mut model = std::mem::take(&mut ctx.skill_curse_model);

    mamodel_load(ctx, &mut model, b"skill_curse.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.skill_curse_anim);

    maanim_load(ctx, &mut anim, b"skill_curse.maanim")?;
    ctx.skill_curse_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_curse_model = model;

    let mut model = std::mem::take(&mut ctx.skill_curse_e_model);

    mamodel_load(ctx, &mut model, b"skill_curse_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.skill_curse_e_anim);

    maanim_load(ctx, &mut anim, b"skill_curse_e.maanim")?;
    ctx.skill_curse_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.skill_curse_e_model = model;

    let mut model = std::mem::take(&mut ctx.wave_attack_model);

    mamodel_load(ctx, &mut model, b"skill_wave_attack.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.wave_anim);

    maanim_load(ctx, &mut anim, b"skill_wave_attack.maanim")?;
    ctx.wave_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.wave_attack_model = model;

    let mut model = std::mem::take(&mut ctx.wave_attack_e_model);

    mamodel_load(ctx, &mut model, b"skill_wave_attack_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.wave_attack_e_anim);

    maanim_load(ctx, &mut anim, b"skill_wave_attack_e.maanim")?;
    ctx.wave_attack_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.wave_attack_e_model = model;

    let mut model = std::mem::take(&mut ctx.smallwave_attack_model);

    mamodel_load(ctx, &mut model, b"skill_smallwave_attack.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.mini_wave_anim);

    maanim_load(ctx, &mut anim, b"skill_smallwave_attack.maanim")?;
    ctx.mini_wave_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.smallwave_attack_model = model;

    let mut model = std::mem::take(&mut ctx.smallwave_attack_e_model);

    mamodel_load(ctx, &mut model, b"skill_smallwave_attack_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.smallwave_attack_e_anim);

    maanim_load(ctx, &mut anim, b"skill_smallwave_attack_e.maanim")?;
    ctx.smallwave_attack_e_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.smallwave_attack_e_model = model;

    let mut explosion = std::mem::take(&mut ctx.explosion_model);

    mamodel_load(ctx, &mut explosion, b"skill_explosion.mamodel")?;

    let mut explosion_e = std::mem::take(&mut ctx.explosion_e_model);

    mamodel_load(ctx, &mut explosion_e, b"skill_explosion_e.mamodel")?;
    mamodel_set_sheet_table(&mut explosion, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut explosion, 0)?;
    mamodel_set_sheet_table(&mut explosion_e, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut explosion_e, 0)?;
    ctx.explosion_model = explosion;
    ctx.explosion_e_model = explosion_e;

    for index in 0..2 {
        let name = string_format_int(ctx, b"skill_explosion%02d.maanim", index)?;
        let mut anim = std::mem::take(&mut ctx.explosion_anims[index as usize]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.explosion_anims[index as usize] = anim;
    }

    let barrier_suffixes: [&[u8]; 3] = [b"", b"_destruction", b"_breaker"];
    let mut model = std::mem::take(&mut ctx.barrier_model);

    mamodel_load(ctx, &mut model, b"skill_barrier_e.mamodel")?;

    for (index, suffix) in barrier_suffixes.iter().enumerate() {
        let name = format_localized(ctx, b"skill_barrier_e%@.maanim", suffix)?;
        let mut anim = std::mem::take(&mut ctx.barrier_anims[index]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.barrier_anims[index] = anim;
    }

    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.barrier_model = model;

    let shield_suffixes: [&[u8]; 5] = [b"00", b"01", b"_revive", b"_destruction", b"_breaker"];
    let mut model = std::mem::take(&mut ctx.demonshield_model);

    mamodel_load(ctx, &mut model, b"skill_demonshield.mamodel")?;

    for (index, suffix) in shield_suffixes.iter().enumerate() {
        let name = format_localized(ctx, b"skill_demonshield%@.maanim", suffix)?;
        let mut anim = std::mem::take(&mut ctx.shield_anims[index]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.shield_anims[index] = anim;
    }

    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.demonshield_model = model;

    let mut model = std::mem::take(&mut ctx.guard_e_model);

    mamodel_load(ctx, &mut model, b"skill_guard_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.guard_e_anim);

    maanim_load(ctx, &mut anim, b"skill_guard_e.maanim")?;
    ctx.guard_e_anim = anim;

    let mut anim = std::mem::take(&mut ctx.guard_e_breaker_anim);

    maanim_load(ctx, &mut anim, b"skill_guard_e_breaker.maanim")?;
    ctx.guard_e_breaker_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.guard_e_model = model;

    let mut model = std::mem::take(&mut ctx.demonsoul_00_model);

    mamodel_load(ctx, &mut model, b"battle_demonsoul_00.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.death_surge_anims[1]);

    maanim_load(ctx, &mut anim, b"battle_demonsoul_00.maanim")?;
    ctx.death_surge_anims[1] = anim;
    ctx.demonsoul_sheets[1] = texture_cache_load(
        ctx,
        b"battle_demonsoul_00.png",
        b"battle_demonsoul_00.imgcut",
        0x2601,
    )?;
    mamodel_set_sheet(&mut model, ctx.demonsoul_sheets[1].clone());
    mamodel_set_single_sheet(&mut model, 1);
    maanim_initialize(&mut model, 0)?;
    ctx.demonsoul_00_model = model;

    let mut model = std::mem::take(&mut ctx.demonsoul_01_model);

    mamodel_load(ctx, &mut model, b"battle_demonsoul_01.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.death_surge_anims[0]);

    maanim_load(ctx, &mut anim, b"battle_demonsoul_01.maanim")?;
    ctx.death_surge_anims[0] = anim;
    ctx.demonsoul_sheets[0] = texture_cache_load(
        ctx,
        b"battle_demonsoul_01.png",
        b"battle_demonsoul_01.imgcut",
        0x2601,
    )?;
    mamodel_set_sheet(&mut model, ctx.demonsoul_sheets[0].clone());
    mamodel_set_single_sheet(&mut model, 1);
    maanim_initialize(&mut model, 0)?;
    ctx.demonsoul_01_model = model;

    let mut model = std::mem::take(&mut ctx.demonsummon_model);

    mamodel_load(ctx, &mut model, b"skill_demonsummon.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.counter_surge_anims[0]);

    maanim_load(ctx, &mut anim, b"skill_demonsummon.maanim")?;
    ctx.counter_surge_anims[0] = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.demonsummon_model = model;

    let mut model = std::mem::take(&mut ctx.demonsummon_e_model);

    mamodel_load(ctx, &mut model, b"skill_demonsummon_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.counter_surge_anims[1]);

    maanim_load(ctx, &mut anim, b"skill_demonsummon_e.maanim")?;
    ctx.counter_surge_anims[1] = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut model, 0)?;
    ctx.demonsummon_e_model = model;

    let warp_suffixes: [&[u8]; 4] = [b"entrance", b"exit", b"chara_entrance", b"chara_exit"];
    let mut warp = std::mem::take(&mut ctx.warp_model);

    mamodel_load(ctx, &mut warp, b"skill_warp.mamodel")?;

    let mut warp_chara = std::mem::take(&mut ctx.warp_chara_model);

    mamodel_load(ctx, &mut warp_chara, b"skill_warp_chara.mamodel")?;

    for (index, suffix) in warp_suffixes.iter().enumerate() {
        let name = format_localized(ctx, b"skill_warp_%@.maanim", suffix)?;
        let mut anim = std::mem::take(&mut ctx.warp_anims[index]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.warp_anims[index] = anim;
    }

    mamodel_set_sheet_table(&mut warp, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut warp, 0)?;
    mamodel_set_sheet_table(&mut warp_chara, &Rc::clone(&ctx.skill_sheets));
    maanim_initialize(&mut warp_chara, 0)?;
    ctx.warp_model = warp;
    ctx.warp_chara_model = warp_chara;

    let mut model = std::mem::take(&mut ctx.strong_attack_model);

    mamodel_load(ctx, &mut model, b"skill_strong_attack.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.savage_vfx_anim);

    maanim_load(ctx, &mut anim, b"skill_strong_attack.maanim")?;
    ctx.savage_vfx_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    ctx.strong_attack_model = model;

    let mut model = std::mem::take(&mut ctx.attack_invalid_model);

    mamodel_load(ctx, &mut model, b"skill_attack_invalid.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.attack_invalid_anim);

    maanim_load(ctx, &mut anim, b"skill_attack_invalid.maanim")?;
    ctx.attack_invalid_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    ctx.attack_invalid_model = model;

    let mut model = std::mem::take(&mut ctx.percentage_attack_model);

    mamodel_load(ctx, &mut model, b"skill_percentage_attack.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.toxic_vfx_anim);

    maanim_load(ctx, &mut anim, b"skill_percentage_attack.maanim")?;
    ctx.toxic_vfx_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    ctx.percentage_attack_model = model;

    let mut volcano = std::mem::take(&mut ctx.volcano_model);

    mamodel_load(ctx, &mut volcano, b"skill_volcano.mamodel")?;

    let mut volcano_e = std::mem::take(&mut ctx.volcano_e_model);

    mamodel_load(ctx, &mut volcano_e, b"skill_volcano_e.mamodel")?;

    let mut small = std::mem::take(&mut ctx.smallvolcano_model);

    mamodel_load(ctx, &mut small, b"skill_smallvolcano.mamodel")?;

    let mut small_e = std::mem::take(&mut ctx.smallvolcano_e_model);

    mamodel_load(ctx, &mut small_e, b"skill_smallvolcano_e.mamodel")?;

    for index in 0..3 {
        let name = string_format_int(ctx, b"skill_volcano%02d.maanim", index)?;
        let mut anim = std::mem::take(&mut ctx.volcano_anims[index as usize]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.volcano_anims[index as usize] = anim;

        let name = string_format_int(ctx, b"skill_smallvolcano%02d.maanim", index)?;
        let mut anim = std::mem::take(&mut ctx.smallvolcano_anims[index as usize]);

        maanim_load(ctx, &mut anim, &name)?;
        ctx.smallvolcano_anims[index as usize] = anim;
    }

    mamodel_set_sheet_table(&mut volcano, &Rc::clone(&ctx.skill_sheets));
    mamodel_set_sheet_table(&mut small, &Rc::clone(&ctx.skill_sheets));
    mamodel_set_sheet_table(&mut volcano_e, &Rc::clone(&ctx.skill_sheets));
    mamodel_set_sheet_table(&mut small_e, &Rc::clone(&ctx.skill_sheets));
    ctx.volcano_model = volcano;
    ctx.volcano_e_model = volcano_e;
    ctx.smallvolcano_model = small;
    ctx.smallvolcano_e_model = small_e;

    let mut model = std::mem::take(&mut ctx.metal_strong_model);

    mamodel_load(ctx, &mut model, b"skill_metal_strong.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.metal_killer_vfx_anim);

    maanim_load(ctx, &mut anim, b"skill_metal_strong.maanim")?;
    ctx.metal_killer_vfx_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    ctx.metal_strong_model = model;

    let mut model = std::mem::take(&mut ctx.recast_decrease_e_model);

    mamodel_load(ctx, &mut model, b"skill_recast_decrease_e.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.drain_vfx_anim);

    maanim_load(ctx, &mut anim, b"skill_recast_decrease_e.maanim")?;
    ctx.drain_vfx_anim = anim;
    mamodel_set_sheet_table(&mut model, &Rc::clone(&ctx.skill_sheets));
    ctx.recast_decrease_e_model = model;
    ctx.fever_sheet = texture_cache_load(ctx, b"FeverEffect.png", b"FeverEffect.imgcut", 0x2601)?;

    let mut model = std::mem::take(&mut ctx.fever_model);

    mamodel_load(ctx, &mut model, b"FeverEffect.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.fever_anim);

    maanim_load(ctx, &mut anim, b"FeverEffect.maanim")?;
    ctx.fever_anim = anim;
    mamodel_set_sheet(&mut model, ctx.fever_sheet.clone());
    mamodel_set_single_sheet(&mut model, 1);
    ctx.fever_model = model;

    let mut faction = 0;
    let mut cat_side = true;

    loop {
        let slots = if cat_side { 0x15 } else { 0xa };

        for slot in 0..slots {
            if !deck_slot_filled(ctx, faction, slot)? {
                continue;
            }

            let unit_id = get_button_unit_id(ctx, faction, slot)?;
            let form = get_button_unit_form(ctx, faction, slot)?;
            let entry = stat_spawn_animation_type(ctx, faction, unit_id, form)?;

            if entry >= 0 && !ctx.effect_sheets.contains_key(&entry) {
                let png = string_format_int(ctx, b"battle_entry_%03d.png", entry)?;
                let cut = string_format_int(ctx, b"battle_entry_%03d.imgcut", entry)?;
                let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

                ctx.effect_sheets.insert(entry, sheet);

                let path = string_format_int(ctx, b"battle_entry_%03d.mamodel", entry)?;
                let mut model = std::mem::take(ctx.effect_models.entry(entry).or_default());

                mamodel_load(ctx, &mut model, &path)?;
                ctx.effect_models.insert(entry, model);

                let path = string_format_int(ctx, b"battle_entry_%03d.maanim", entry)?;
                let mut anim =
                    std::mem::take(std_map_int_maanim_subscript(&mut ctx.effect_anims, &entry));

                maanim_load(ctx, &mut anim, &path)?;
                *std_map_int_maanim_subscript(&mut ctx.effect_anims, &entry) = anim;

                let table = Rc::from([cell::Cell::new(
                    ctx.effect_sheets.entry(entry).or_default().clone(),
                )]);
                let model = ctx.effect_models.entry(entry).or_default();

                mamodel_set_sheet_table(model, &table);
                mamodel_set_single_sheet(ctx.effect_models.entry(entry).or_default(), 1);
            }

            let soul = stat_soul_animation_type(ctx, faction, unit_id, form)?;
            let key = soul.wrapping_add(0x3e8);

            if soul < 0 || ctx.effect_sheets.contains_key(&key) {
                continue;
            }

            let png = string_format_int(ctx, b"battle_soul_%03d.png", soul)?;
            let cut = string_format_int(ctx, b"battle_soul_%03d.imgcut", soul)?;
            let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

            ctx.effect_sheets.insert(key, sheet);

            let path = string_format_int(ctx, b"battle_soul_%03d.mamodel", soul)?;
            let mut model = std::mem::take(ctx.effect_models.entry(key).or_default());

            mamodel_load(ctx, &mut model, &path)?;
            ctx.effect_models.insert(key, model);

            let path = string_format_int(ctx, b"battle_soul_%03d.maanim", soul)?;
            let mut anim =
                std::mem::take(std_map_int_maanim_subscript(&mut ctx.effect_anims, &key));

            maanim_load(ctx, &mut anim, &path)?;
            *std_map_int_maanim_subscript(&mut ctx.effect_anims, &key) = anim;

            let table = Rc::from([cell::Cell::new(
                ctx.effect_sheets.entry(key).or_default().clone(),
            )]);
            let model = ctx.effect_models.entry(key).or_default();

            mamodel_set_sheet_table(model, &table);
            mamodel_set_single_sheet(ctx.effect_models.entry(key).or_default(), 1);
        }

        if !cat_side {
            break;
        }

        faction = 1;
        cat_side = false;
    }

    ctx.img015_sheet = texture_cache_load(ctx, b"img015.png", b"img015.imgcut", 0x2601)?;
    ctx.equipment_attribute_sheet = texture_cache_load(
        ctx,
        b"equipment_attribute.png",
        b"equipment_attribute.imgcut",
        0x2601,
    )?;
    ctx.equipment_effect_sheet = texture_cache_load(
        ctx,
        b"equipment_effect.png",
        b"equipment_effect.imgcut",
        0x2601,
    )?;
    ctx.equipment_grade_sheet = texture_cache_load(
        ctx,
        b"equipment_grade.png",
        b"equipment_grade.imgcut",
        0x2601,
    )?;
    ctx.equipment_shadow_sheet = texture_cache_load(
        ctx,
        b"equipment_shadow.png",
        b"equipment_shadow.imgcut",
        0x2601,
    )?;
    ctx.equipment_attribute_s_sheet = texture_cache_load(
        ctx,
        b"equipment_attribute_s.png",
        b"equipment_attribute_s.imgcut",
        0x2601,
    )?;
    ctx.equipment_effect_s_sheet = texture_cache_load(
        ctx,
        b"equipment_effect_s.png",
        b"equipment_effect_s.imgcut",
        0x2601,
    )?;

    let mut model = std::mem::take(&mut ctx.invoke_equipment_model);

    mamodel_load(ctx, &mut model, b"invoke_equipment.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.invoke_equipment_anim);

    maanim_load(ctx, &mut anim, b"invoke_equipment.maanim")?;
    ctx.invoke_equipment_anim = anim;
    mamodel_set_sheet(&mut model, ctx.img002_sheet.clone());
    ctx.invoke_equipment_model = model;

    let mut model = std::mem::take(&mut ctx.demonbattle_model);

    mamodel_load(ctx, &mut model, b"demonbattle_000.mamodel")?;

    let mut anim = std::mem::take(&mut ctx.demon_banner_anim);

    maanim_load(ctx, &mut anim, b"demonbattle_000.maanim")?;
    ctx.demon_banner_anim = anim;

    let text = query_localizable(ctx, b"sealed_announce");
    let lines = string_split(&text, b"<br>");
    let font = ctx.default_font.clone();
    let first = lines.first().ok_or(Fault::NullPointer { site: SITE })?;
    let label = get_text_texture(text_texture_cache(ctx)?, first, &font, 0x1e, 0, 0);

    ctx.sealed_announce_sheets[0] = Some(Rc::new(Imgcut {
        label: label.id,
        width: label.width,
        height: label.height,
        whole: 1,
        ..Default::default()
    }));

    let enemy = get_castle_enemy_row(ctx)?.wrapping_add(-2);
    let cap = get_altar_level_cap(ctx, enemy)?.wrapping_add(1);
    let second = lines.get(1).ok_or(Fault::IndexOutOfRange {
        site: SITE,
        index: 1,
        limit: lines.len() as i64,
    })?;
    let line = string_format_int(ctx, second, cap)?;
    let label = get_text_texture(text_texture_cache(ctx)?, &line, &font, 0x1e, 0, 0);

    ctx.sealed_announce_sheets[1] = Some(Rc::new(Imgcut {
        label: label.id,
        width: label.width,
        height: label.height,
        whole: 1,
        ..Default::default()
    }));
    ctx.sealed_announce_sheets[2] = ctx.img002_sheet.clone();
    mamodel_set_sheet_table(
        &mut model,
        &Rc::from([
            cell::Cell::new(ctx.sealed_announce_sheets[0].clone()),
            cell::Cell::new(ctx.sealed_announce_sheets[1].clone()),
            cell::Cell::new(ctx.sealed_announce_sheets[2].clone()),
        ]),
    );

    let part = mamodel_get_part(&model, 0).ok_or(Fault::NullPointer { site: SITE })?;

    model
        .parts
        .get_mut(part + 7)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: (part + 7) as i64,
            limit: 0,
        })?
        .set_i32_at(0x24, 0);

    let part = mamodel_get_part(&model, 0).ok_or(Fault::NullPointer { site: SITE })?;

    model
        .parts
        .get_mut(part + 8)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: (part + 8) as i64,
            limit: 0,
        })?
        .set_i32_at(0x24, 1);

    let width = imgcut_get_width(
        ctx.sealed_announce_sheets[0]
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
    );
    let part = mamodel_get_part(&model, 0).ok_or(Fault::NullPointer { site: SITE })?;
    let target = model
        .parts
        .get_mut(part + 7)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: (part + 7) as i64,
            limit: 0,
        })?;

    target.set_i32_at(0x3c, target.i32_at(0x3c).wrapping_sub(width / 2));

    let width = imgcut_get_width(
        ctx.sealed_announce_sheets[1]
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
    );
    let part = mamodel_get_part(&model, 0).ok_or(Fault::NullPointer { site: SITE })?;
    let target = model
        .parts
        .get_mut(part + 8)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: (part + 8) as i64,
            limit: 0,
        })?;

    target.set_i32_at(0x3c, target.i32_at(0x3c).wrapping_sub(width / 2));
    ctx.demonbattle_model = model;

    Ok(())
}
