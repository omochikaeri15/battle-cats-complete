use crate::{Fault, ops};

use super::{
    AppContext, ENEMY_STATS, ENEMY_STATS_STRIDE, EnemyStats, Entity, call_rng, compute_attack,
    compute_hp, get_base_pos_x, get_castle_id, get_castle_row, get_cat_combo_bonus,
    get_counter_surge, get_entity_base_idx, get_max_hp, get_orb_value_max, get_orb_value_sum,
    get_setting, get_spawn_anim_flag, get_spawn_anim_type, get_stat_range, get_talent_value,
    has_orb, is_boss, orb_deploy_condition_slot, read_flag, set_area_attack, set_attack_abilities,
    set_attack_cooldown, set_attack_damage, set_attack_end_mode, set_attack_foreswing,
    set_attack_interval, set_attack_only, set_attacks_remaining, set_barrier_breaker_chance,
    set_barrier_hp, set_barrier_vfx_active, set_base_destroyer, set_behemoth_dodge_chance,
    set_behemoth_dodge_duration, set_behemoth_dodge_timer, set_behemoth_slayer, set_boss_type,
    set_boss_wave_immune, set_burrow_count, set_burrow_distance, set_cannon_blast_hit,
    set_cannon_charge_orb, set_cannon_hit_stamp, set_cash_back_pct, set_colossus_orb_pcts,
    set_colossus_slayer, set_conjure_deck_slot, set_conjure_unit_id, set_counter_surge,
    set_counter_surge_once, set_counter_surge_pct, set_crit_vfx, set_critical_chance,
    set_curse_chance, set_curse_duration, set_curse_immune, set_curse_length, set_curse_resist_pct,
    set_curse_timer, set_death_surge_anchor, set_death_surge_chance, set_death_surge_level,
    set_death_surge_mini, set_death_surge_pct, set_death_surge_span, set_death_timer,
    set_dodge_chance, set_dodge_duration, set_dodge_timer, set_double_bounty, set_drain_chance,
    set_drain_immune, set_drain_pct, set_drain_percent, set_entity_frame, set_entity_state,
    set_eva_killer, set_explosion_anchor, set_explosion_chance, set_explosion_immune,
    set_explosion_resist_pct, set_explosion_span, set_first_bounty_pct, set_frame_damage,
    set_freeze_chance, set_freeze_duration, set_freeze_immune, set_freeze_length,
    set_freeze_resist_pct, set_freeze_timer, set_gudetama_soul, set_hitbox_pos, set_hitbox_width,
    set_hp, set_insane_damage, set_insanely_tough, set_kb_proc_hit, set_kill_count,
    set_knockback_chance, set_knockback_immune, set_knockback_resist_pct, set_knockbacks,
    set_ld_anchor, set_ld_flag, set_ld_span, set_massive_damage, set_max_hp, set_metal,
    set_metal_killer_pct, set_metal_killer_vfx, set_mini_surge, set_no_revive, set_occupant,
    set_orb_dodge_chance, set_orb_dodge_duration, set_orb_dodge_timer, set_paid_cost, set_pos_x,
    set_pos_y, set_prev_curse_timer, set_prev_freeze_timer, set_prev_slow_timer,
    set_prev_weaken_timer, set_proc_badge, set_resist, set_revive_count, set_revive_hp,
    set_revive_time, set_revive_timer, set_sage_slayer, set_savage_blow_boost,
    set_savage_blow_chance, set_savage_blow_vfx, set_score_value, set_shield_hp, set_shield_max,
    set_shield_pierce_chance, set_shield_regen, set_shield_vfx, set_shockwave_counter,
    set_slow_chance, set_slow_duration, set_slow_immune, set_slow_length, set_slow_resist_pct,
    set_slow_timer, set_soul_anim_type, set_soulstrike, set_spawn_anim_flag, set_spawn_anim_type,
    set_spawn_serial, set_speed, set_standing_range, set_strengthen_boost,
    set_strengthen_threshold, set_strong_against, set_surge_anchor, set_surge_chance,
    set_surge_immune, set_surge_level, set_surge_resist_pct, set_surge_span, set_survive_chance,
    set_total_damage_taken, set_toxic_chance, set_toxic_damage, set_toxic_immune,
    set_toxic_resist_pct, set_toxic_vfx, set_trait_aku, set_trait_alien, set_trait_angel,
    set_trait_behemoth, set_trait_colossus, set_trait_dark, set_trait_dojo, set_trait_eva_angel,
    set_trait_floating, set_trait_kaijin, set_trait_metal, set_trait_red, set_trait_relic,
    set_trait_sage, set_trait_starred_alien, set_trait_traitless, set_trait_witch,
    set_trait_zombie, set_warp_anchor, set_warp_chance, set_warp_duration, set_warp_immune,
    set_warp_resist_pct, set_warp_span, set_warp_timer, set_wave_block, set_wave_chance,
    set_wave_immune, set_wave_level, set_wave_mini, set_wave_resist_pct, set_weaken_active,
    set_weaken_active_pct, set_weaken_chance, set_weaken_duration, set_weaken_immune,
    set_weaken_pct, set_weaken_resist_pct, set_weaken_timer, set_witch_slayer, set_zkill_hit,
    set_zombie_killer, slot_occupied, stage_entry_boss, stage_entry_col10, stat_area_attack,
    stat_attack_cooldown, stat_attack_count_state, stat_attack_count_total, stat_attack_foreswing,
    stat_attack_has_abilities, stat_attack_has_ld, stat_attack_ld_anchor, stat_attack_ld_span,
    stat_attack_only, stat_barrier_breaker_chance, stat_barrier_hitpoints, stat_base_destroyer,
    stat_behemoth_dodge_chance, stat_behemoth_dodge_duration, stat_behemoth_slayer,
    stat_boss_wave_immune, stat_colossus_slayer, stat_conjure_unit_id, stat_counter_surge,
    stat_critical_chance, stat_curse_chance, stat_curse_duration, stat_curse_immune,
    stat_death_surge_anchor, stat_death_surge_chance, stat_death_surge_level,
    stat_death_surge_span, stat_dodge_chance, stat_dodge_duration, stat_double_bounty,
    stat_drain_chance, stat_drain_immune, stat_drain_percent, stat_eva_killer,
    stat_explosion_chance, stat_explosion_immune, stat_explosion_spawn_anchor,
    stat_explosion_spawn_span, stat_freeze_chance, stat_freeze_duration, stat_freeze_immune,
    stat_hitbox_position, stat_hitbox_width, stat_insane_damage, stat_insanely_tough,
    stat_is_metal, stat_knockback_chance, stat_knockback_immune, stat_knockbacks,
    stat_massive_damage, stat_metal_killer_percent, stat_mini_surge_flag, stat_mini_wave_flag,
    stat_resist, stat_sage_slayer, stat_savage_blow_boost, stat_savage_blow_chance,
    stat_shield_hitpoints, stat_shield_pierce_chance, stat_shield_regen, stat_slow_chance,
    stat_slow_duration, stat_slow_immune, stat_soul_animation_type, stat_soulstrike,
    stat_spawn_animation_flag, stat_spawn_animation_type, stat_speed, stat_strengthen_boost,
    stat_strengthen_threshold, stat_strong_against, stat_surge_chance, stat_surge_immune,
    stat_surge_level, stat_surge_spawn_anchor, stat_surge_spawn_span, stat_survive,
    stat_time_before_death, stat_toxic_chance, stat_toxic_damage, stat_toxic_immune,
    stat_use_gudetama_soul, stat_warp_chance, stat_warp_dist_anchor, stat_warp_dist_span,
    stat_warp_duration, stat_warp_immune, stat_wave_block, stat_wave_chance, stat_wave_immune,
    stat_wave_level, stat_weaken_chance, stat_weaken_duration, stat_weaken_immune, stat_weaken_to,
    stat_witch_killer, stat_zombie_killer, trait_aku, trait_alien, trait_angel, trait_behemoth,
    trait_colossus, trait_dark, trait_dojo, trait_eva, trait_floating, trait_kaijin, trait_metal,
    trait_red, trait_relic, trait_sage, trait_starred_alien, trait_traitless, trait_witch,
    trait_zombie,
};

pub fn spawn_entity(
    ctx: &mut AppContext,
    faction: i32,
    row: i32,
    level: i32,
    z_min: i32,
    z_max: i32,
    form: i32,
    mag_slot: i32,
) -> Result<i32, Fault> {
    if row == -1 {
        return Ok(-2);
    }

    let unit_id = row.wrapping_add(-2);
    let mut slot = 1i32;

    loop {
        if slot_occupied(ctx, faction, slot)? == 0 {
            break;
        }

        slot += 1;

        if slot == 0x33 {
            return Ok(-1);
        }
    }

    set_occupant(ctx, faction, slot, 2, unit_id)?;
    set_entity_frame(ctx, faction, slot, 0)?;

    if faction == 1 && get_entity_base_idx(ctx)? == slot {
        set_pos_x(ctx, 1, slot, 0xc80)?;
    } else {
        let far_side = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0xaf0);

        set_pos_x(
            ctx,
            faction,
            slot,
            if faction == 0 { far_side } else { 0xaf0 },
        )?;
    }

    set_pos_y(ctx, faction, slot, 0x1180)?;

    let z_layer = AppContext::entity_field(faction, slot, Entity::Z_LAYER);
    ctx.set_i32_at(z_layer, z_min)?;

    let z_range = z_max.wrapping_sub(z_min);
    let z_roll = call_rng(ctx, (if z_range > 0 { z_range } else { 0 }).wrapping_add(1));

    ctx.set_i32_at(z_layer, ctx.i32_at(z_layer)?.wrapping_add(z_roll))?;
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::LEVEL),
        level,
    )?;

    set_attack_cooldown(ctx, faction, slot, 0)?;
    set_frame_damage(ctx, faction, slot, 0)?;
    set_drain_pct(ctx, faction, slot, 0)?;
    set_cannon_hit_stamp(ctx, faction, slot, 0)?;
    set_cannon_blast_hit(ctx, faction, slot, 0)?;
    set_total_damage_taken(ctx, faction, slot, 0)?;
    let attacks_remaining = stat_attack_count_total(ctx, faction, unit_id, form)?;
    set_attacks_remaining(ctx, faction, slot, attacks_remaining)?;
    let death_timer = stat_time_before_death(ctx, faction, unit_id, form)?;
    set_death_timer(ctx, faction, slot, death_timer)?;
    let spawn_anim_type = stat_spawn_animation_type(ctx, faction, unit_id, form)?;
    set_spawn_anim_type(ctx, faction, slot, spawn_anim_type)?;
    let soul_anim_type = stat_soul_animation_type(ctx, faction, unit_id, form)?;
    set_soul_anim_type(ctx, faction, slot, soul_anim_type)?;
    let spawn_anim_flag = stat_spawn_animation_flag(ctx, faction, unit_id, form)?;
    set_spawn_anim_flag(ctx, faction, slot, spawn_anim_flag as i32)?;
    let gudetama_soul = stat_use_gudetama_soul(ctx, faction, unit_id, form)?;
    set_gudetama_soul(ctx, faction, slot, gudetama_soul as i32)?;

    let level = level.wrapping_add(1);

    let max_hp = compute_hp(ctx, faction, unit_id, form, level, mag_slot)?;
    set_max_hp(ctx, faction, slot, max_hp)?;
    let hp = get_max_hp(ctx, faction, slot)?;
    set_hp(ctx, faction, slot, hp)?;
    let knockbacks = stat_knockbacks(ctx, faction, unit_id, form)?;
    set_knockbacks(ctx, faction, slot, knockbacks)?;
    let speed = stat_speed(ctx, faction, unit_id, form)?;
    set_speed(ctx, faction, slot, speed)?;
    let attack_interval = stat_attack_cooldown(ctx, faction, unit_id, form)?;
    set_attack_interval(ctx, faction, slot, attack_interval)?;
    let standing_range = get_stat_range(ctx, faction, unit_id, form)?;
    set_standing_range(ctx, faction, slot, standing_range)?;
    let hitbox_pos = stat_hitbox_position(ctx, faction, unit_id, form)?;
    set_hitbox_pos(ctx, faction, slot, hitbox_pos)?;
    let hitbox_width = stat_hitbox_width(ctx, faction, unit_id, form)?;
    set_hitbox_width(ctx, faction, slot, hitbox_width)?;
    let attack_end_mode = stat_attack_count_state(ctx, faction, unit_id, form)?;
    set_attack_end_mode(ctx, faction, slot, attack_end_mode)?;

    let skip_mult = if faction == 1 {
        (slot == get_entity_base_idx(ctx)?) as u8
    } else {
        0
    };

    let attack_damage = compute_attack(
        ctx, faction, unit_id, form, level, mag_slot, 0, 0, skip_mult,
    )?;
    set_attack_damage(ctx, faction, slot, 0, attack_damage)?;
    let attack_foreswing = stat_attack_foreswing(ctx, faction, unit_id, form, 0)?;
    set_attack_foreswing(ctx, faction, slot, 0, attack_foreswing)?;
    let attack_abilities = stat_attack_has_abilities(ctx, faction, unit_id, form, 0)?;
    set_attack_abilities(ctx, faction, slot, 0, attack_abilities as i32)?;

    let skip_mult = if faction == 1 {
        (slot == get_entity_base_idx(ctx)?) as u8
    } else {
        0
    };

    let attack_damage = compute_attack(
        ctx, faction, unit_id, form, level, mag_slot, 1, 0, skip_mult,
    )?;
    set_attack_damage(ctx, faction, slot, 1, attack_damage)?;
    let attack_foreswing = stat_attack_foreswing(ctx, faction, unit_id, form, 1)?;
    set_attack_foreswing(ctx, faction, slot, 1, attack_foreswing)?;
    let attack_abilities = stat_attack_has_abilities(ctx, faction, unit_id, form, 1)?;
    set_attack_abilities(ctx, faction, slot, 1, attack_abilities as i32)?;

    let skip_mult = if faction == 1 {
        (slot == get_entity_base_idx(ctx)?) as u8
    } else {
        0
    };

    let attack_damage = compute_attack(
        ctx, faction, unit_id, form, level, mag_slot, 2, 0, skip_mult,
    )?;
    set_attack_damage(ctx, faction, slot, 2, attack_damage)?;
    let attack_foreswing = stat_attack_foreswing(ctx, faction, unit_id, form, 2)?;
    set_attack_foreswing(ctx, faction, slot, 2, attack_foreswing)?;
    let attack_abilities = stat_attack_has_abilities(ctx, faction, unit_id, form, 2)?;
    set_attack_abilities(ctx, faction, slot, 2, attack_abilities as i32)?;

    let trait_red = trait_red(ctx, faction, unit_id, form, 1)?;
    set_trait_red(ctx, faction, slot, trait_red as i32)?;
    let trait_floating = trait_floating(ctx, faction, unit_id, form, 1)?;
    set_trait_floating(ctx, faction, slot, trait_floating as i32)?;
    let trait_dark = trait_dark(ctx, faction, unit_id, form, 1)?;
    set_trait_dark(ctx, faction, slot, trait_dark as i32)?;
    let trait_metal = trait_metal(ctx, faction, unit_id, form, 1)?;
    set_trait_metal(ctx, faction, slot, trait_metal as i32)?;
    let metal = stat_is_metal(ctx, faction, unit_id, form)?;
    set_metal(ctx, faction, slot, metal as i32)?;
    let trait_traitless = trait_traitless(ctx, faction, unit_id, form, 1)?;
    set_trait_traitless(ctx, faction, slot, trait_traitless as i32)?;
    let trait_angel = trait_angel(ctx, faction, unit_id, form, 1)?;
    set_trait_angel(ctx, faction, slot, trait_angel as i32)?;
    let trait_alien = trait_alien(ctx, faction, unit_id, form, 1)?;
    set_trait_alien(ctx, faction, slot, trait_alien as i32)?;
    let trait_starred_alien = trait_starred_alien(ctx, faction, unit_id)?;
    set_trait_starred_alien(ctx, faction, slot, trait_starred_alien)?;
    let trait_zombie = trait_zombie(ctx, faction, unit_id, form, 1)?;
    set_trait_zombie(ctx, faction, slot, trait_zombie as i32)?;
    let trait_witch = trait_witch(ctx, faction, unit_id, form, 1)?;
    set_trait_witch(ctx, faction, slot, trait_witch as i32)?;
    let trait_eva_angel = trait_eva(ctx, faction, unit_id, form, 1)?;
    set_trait_eva_angel(ctx, faction, slot, trait_eva_angel as i32)?;
    let trait_relic = trait_relic(ctx, faction, unit_id, form, 1)?;
    set_trait_relic(ctx, faction, slot, trait_relic as i32)?;
    let trait_aku = trait_aku(ctx, faction, unit_id, form, 1)?;
    set_trait_aku(ctx, faction, slot, trait_aku as i32)?;
    let trait_colossus = trait_colossus(ctx, faction, unit_id)?;
    set_trait_colossus(ctx, faction, slot, trait_colossus as i32)?;
    let trait_behemoth = trait_behemoth(ctx, faction, unit_id)?;
    set_trait_behemoth(ctx, faction, slot, trait_behemoth as i32)?;
    let trait_sage = trait_sage(ctx, faction, unit_id)?;
    set_trait_sage(ctx, faction, slot, trait_sage as i32)?;
    let trait_kaijin = trait_kaijin(ctx, faction, unit_id)?;
    set_trait_kaijin(ctx, faction, slot, trait_kaijin as i32)?;
    let area_attack = stat_area_attack(ctx, faction, unit_id, form)?;
    set_area_attack(ctx, faction, slot, area_attack as i32)?;
    let wave_mini = stat_mini_wave_flag(ctx, faction, unit_id, form)?;
    set_wave_mini(ctx, faction, slot, wave_mini)?;
    let wave_chance = stat_wave_chance(ctx, faction, unit_id, form, -1)?;
    set_wave_chance(ctx, faction, slot, wave_chance)?;
    let wave_level = stat_wave_level(ctx, faction, unit_id, form, -1)?;
    set_wave_level(ctx, faction, slot, wave_level)?;
    let knockback_chance = stat_knockback_chance(ctx, faction, unit_id, form)?;
    set_knockback_chance(ctx, faction, slot, knockback_chance)?;
    let attack_only = stat_attack_only(ctx, faction, unit_id, form)?;
    set_attack_only(ctx, faction, slot, attack_only as u8)?;
    let double_bounty = stat_double_bounty(ctx, faction, unit_id, form)?;
    set_double_bounty(ctx, faction, slot, double_bounty as u8)?;

    let low = form < 2 || faction != 0;
    let mut cash_back = 0i32;

    if !low && orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)? {
        cash_back = get_orb_value_sum(ctx, unit_id, 7, 0, 0)?;
    }

    set_cash_back_pct(ctx, faction, slot, cash_back)?;
    let survive_chance = stat_survive(ctx, faction, unit_id, form)?;
    set_survive_chance(ctx, faction, slot, survive_chance)?;
    let ld_flag = stat_attack_has_ld(ctx, faction, unit_id, form, 0)?;
    set_ld_flag(ctx, faction, slot, 0, ld_flag as u8)?;
    let ld_anchor = stat_attack_ld_anchor(ctx, faction, unit_id, form, 0)?;
    set_ld_anchor(ctx, faction, slot, 0, ld_anchor)?;
    let ld_span = stat_attack_ld_span(ctx, faction, unit_id, form, 0)?;
    set_ld_span(ctx, faction, slot, 0, ld_span)?;
    let ld_flag = stat_attack_has_ld(ctx, faction, unit_id, form, 1)?;
    set_ld_flag(ctx, faction, slot, 1, ld_flag as u8)?;
    let ld_anchor = stat_attack_ld_anchor(ctx, faction, unit_id, form, 1)?;
    set_ld_anchor(ctx, faction, slot, 1, ld_anchor)?;
    let ld_span = stat_attack_ld_span(ctx, faction, unit_id, form, 1)?;
    set_ld_span(ctx, faction, slot, 1, ld_span)?;
    let ld_flag = stat_attack_has_ld(ctx, faction, unit_id, form, 2)?;
    set_ld_flag(ctx, faction, slot, 2, ld_flag as u8)?;
    let ld_anchor = stat_attack_ld_anchor(ctx, faction, unit_id, form, 2)?;
    set_ld_anchor(ctx, faction, slot, 2, ld_anchor)?;
    let ld_span = stat_attack_ld_span(ctx, faction, unit_id, form, 2)?;
    set_ld_span(ctx, faction, slot, 2, ld_span)?;
    let warp_chance = stat_warp_chance(ctx, faction, unit_id, form)?;
    set_warp_chance(ctx, faction, slot, warp_chance)?;
    let warp_duration = stat_warp_duration(ctx, faction, unit_id, form)?;
    set_warp_duration(ctx, faction, slot, warp_duration)?;
    let warp_anchor = stat_warp_dist_anchor(ctx, faction, unit_id, form)?;
    set_warp_anchor(ctx, faction, slot, warp_anchor)?;
    let warp_span = stat_warp_dist_span(ctx, faction, unit_id, form)?;
    set_warp_span(ctx, faction, slot, warp_span)?;
    let toxic_chance = stat_toxic_chance(ctx, faction, unit_id)?;
    set_toxic_chance(ctx, faction, slot, toxic_chance)?;
    let toxic_damage = stat_toxic_damage(ctx, faction, unit_id)?;
    set_toxic_damage(ctx, faction, slot, toxic_damage)?;
    let surge_chance = stat_surge_chance(ctx, faction, unit_id, form)?;
    set_surge_chance(ctx, faction, slot, surge_chance)?;
    let surge_anchor = stat_surge_spawn_anchor(ctx, faction, unit_id, form)?;
    set_surge_anchor(ctx, faction, slot, surge_anchor)?;
    let surge_span = stat_surge_spawn_span(ctx, faction, unit_id, form)?;
    set_surge_span(ctx, faction, slot, surge_span)?;
    let surge_level = stat_surge_level(ctx, faction, unit_id, form)?;
    set_surge_level(ctx, faction, slot, surge_level)?;
    let mini_surge = stat_mini_surge_flag(ctx, faction, unit_id, form)?;
    set_mini_surge(ctx, faction, slot, mini_surge)?;
    let death_surge_chance = stat_death_surge_chance(ctx, faction, unit_id)?;
    set_death_surge_chance(ctx, faction, slot, death_surge_chance)?;
    let death_surge_anchor = stat_death_surge_anchor(ctx, faction, unit_id)?;
    set_death_surge_anchor(ctx, faction, slot, death_surge_anchor)?;
    let death_surge_span = stat_death_surge_span(ctx, faction, unit_id)?;
    set_death_surge_span(ctx, faction, slot, death_surge_span)?;
    let death_surge_level = stat_death_surge_level(ctx, faction, unit_id)?;
    set_death_surge_level(ctx, faction, slot, death_surge_level)?;
    set_death_surge_pct(ctx, faction, slot, 0x64)?;
    set_death_surge_mini(ctx, faction, slot, 0)?;

    if !low {
        if orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)? {
            let death_surge = get_orb_value_max(ctx, unit_id, 5, 0, 0)?;

            if death_surge > 0 {
                set_death_surge_chance(ctx, 0, slot, 0x64)?;
                let distance =
                    get_setting(&ctx.settings, b"battle_af_death_volcano_distance", 0x258)?;
                set_death_surge_anchor(ctx, 0, slot, distance)?;
                let width = get_setting(&ctx.settings, b"battle_af_death_volcano_width", 0xfa0)?;
                set_death_surge_span(ctx, 0, slot, width)?;
                set_death_surge_level(ctx, 0, slot, 1)?;
                set_death_surge_pct(ctx, 0, slot, death_surge.wrapping_mul(5))?;
                set_death_surge_mini(ctx, 0, slot, 1)?;
            }
        }

        let counter_surge = stat_counter_surge(ctx, 0, unit_id, form)?;
        set_counter_surge(ctx, 0, slot, counter_surge as i32)?;
        set_counter_surge_pct(ctx, 0, slot, 0x64)?;
        set_counter_surge_once(ctx, 0, slot, 0)?;

        if !get_counter_surge(ctx, 0, slot)?
            && orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)?
        {
            let counter = get_orb_value_max(ctx, unit_id, 0x11, 0, 0)?;

            if counter > 0 {
                set_counter_surge(ctx, 0, slot, 1)?;
                set_counter_surge_pct(ctx, 0, slot, counter)?;
                set_counter_surge_once(ctx, 0, slot, 1)?;
            }
        }
    } else {
        let counter_surge = stat_counter_surge(ctx, faction, unit_id, form)?;
        set_counter_surge(ctx, faction, slot, counter_surge as i32)?;
        set_counter_surge_pct(ctx, faction, slot, 0x64)?;
        set_counter_surge_once(ctx, faction, slot, 0)?;
    }

    let explosion_chance = stat_explosion_chance(ctx, faction, unit_id, form)?;
    set_explosion_chance(ctx, faction, slot, explosion_chance)?;
    let explosion_anchor = stat_explosion_spawn_anchor(ctx, faction, unit_id, form)?;
    set_explosion_anchor(ctx, faction, slot, explosion_anchor)?;
    let explosion_span = stat_explosion_spawn_span(ctx, faction, unit_id, form)?;
    set_explosion_span(ctx, faction, slot, explosion_span)?;
    let conjure_unit_id = stat_conjure_unit_id(ctx, faction, unit_id, form)?;
    set_conjure_unit_id(ctx, faction, slot, conjure_unit_id)?;
    set_conjure_deck_slot(ctx, faction, slot, -1)?;
    let freeze_chance = stat_freeze_chance(ctx, faction, unit_id, form)?;
    set_freeze_chance(ctx, faction, slot, freeze_chance)?;
    let freeze_duration = stat_freeze_duration(ctx, faction, unit_id, form)?;
    set_freeze_duration(ctx, faction, slot, freeze_duration)?;
    let slow_chance = stat_slow_chance(ctx, faction, unit_id, form)?;
    set_slow_chance(ctx, faction, slot, slow_chance)?;
    let slow_duration = stat_slow_duration(ctx, faction, unit_id, form)?;
    set_slow_duration(ctx, faction, slot, slow_duration)?;
    let weaken_chance = stat_weaken_chance(ctx, faction, unit_id, form)?;
    set_weaken_chance(ctx, faction, slot, weaken_chance)?;
    let weaken_duration = stat_weaken_duration(ctx, faction, unit_id, form)?;
    set_weaken_duration(ctx, faction, slot, weaken_duration)?;
    let weaken_pct = stat_weaken_to(ctx, faction, unit_id, form)?;
    set_weaken_pct(ctx, faction, slot, weaken_pct)?;
    let curse_chance = stat_curse_chance(ctx, faction, unit_id, form)?;
    set_curse_chance(ctx, faction, slot, curse_chance)?;
    let curse_duration = stat_curse_duration(ctx, faction, unit_id, form)?;
    set_curse_duration(ctx, faction, slot, curse_duration)?;
    let strengthen_threshold = stat_strengthen_threshold(ctx, faction, unit_id, form)?;
    set_strengthen_threshold(ctx, faction, slot, strengthen_threshold)?;
    let strengthen_boost = stat_strengthen_boost(ctx, faction, unit_id, form)?;
    set_strengthen_boost(ctx, faction, slot, strengthen_boost)?;
    let strong_against = stat_strong_against(ctx, faction, unit_id, form)?;
    set_strong_against(ctx, faction, slot, strong_against as u8)?;
    let resist = stat_resist(ctx, faction, unit_id, form)?;
    set_resist(ctx, faction, slot, resist as u8)?;
    let massive_damage = stat_massive_damage(ctx, faction, unit_id, form)?;
    set_massive_damage(ctx, faction, slot, massive_damage as u8)?;
    let insanely_tough = stat_insanely_tough(ctx, faction, unit_id, form)?;
    set_insanely_tough(ctx, faction, slot, insanely_tough as u8)?;
    let insane_damage = stat_insane_damage(ctx, faction, unit_id, form)?;
    set_insane_damage(ctx, faction, slot, insane_damage as u8)?;
    let critical_chance = stat_critical_chance(ctx, faction, unit_id, form)?;
    set_critical_chance(ctx, faction, slot, critical_chance)?;
    let base_destroyer = stat_base_destroyer(ctx, faction, unit_id, form)?;
    set_base_destroyer(ctx, faction, slot, base_destroyer as i32)?;
    let zombie_killer = stat_zombie_killer(ctx, faction, unit_id, form)?;
    set_zombie_killer(ctx, faction, slot, zombie_killer as u8)?;
    let witch_slayer = stat_witch_killer(ctx, faction, unit_id, form)?;
    set_witch_slayer(ctx, faction, slot, witch_slayer as u8)?;
    let eva_killer = stat_eva_killer(ctx, faction, unit_id, form)?;
    set_eva_killer(ctx, faction, slot, eva_killer as u8)?;

    let colossus_slayer = stat_colossus_slayer(ctx, faction, unit_id, form)?;

    set_colossus_slayer(ctx, faction, slot, colossus_slayer as u8)?;
    set_colossus_orb_pcts(ctx, faction, slot, 0, 0)?;

    if !colossus_slayer
        && !low
        && orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)?
    {
        let attack_pct = get_orb_value_max(ctx, unit_id, 0xa, 0, 0)?;
        let defense_pct = get_orb_value_max(ctx, unit_id, 0xa, 1, 0)?;

        if attack_pct > 0 && defense_pct > 0 {
            set_colossus_slayer(ctx, 0, slot, 1)?;
            set_colossus_orb_pcts(ctx, 0, slot, attack_pct, defense_pct)?;
        }
    }

    let barrier_breaker_chance = stat_barrier_breaker_chance(ctx, faction, unit_id, form)?;
    set_barrier_breaker_chance(ctx, faction, slot, barrier_breaker_chance)?;
    let shield_pierce_chance = stat_shield_pierce_chance(ctx, faction, unit_id, form)?;
    set_shield_pierce_chance(ctx, faction, slot, shield_pierce_chance)?;
    let savage_blow_chance = stat_savage_blow_chance(ctx, faction, unit_id, form)?;
    set_savage_blow_chance(ctx, faction, slot, savage_blow_chance)?;
    let savage_blow_boost = stat_savage_blow_boost(ctx, faction, unit_id, form)?;
    set_savage_blow_boost(ctx, faction, slot, savage_blow_boost)?;
    let soulstrike = stat_soulstrike(ctx, faction, unit_id, form)?;
    set_soulstrike(ctx, faction, slot, soulstrike as i32)?;
    let behemoth_slayer = stat_behemoth_slayer(ctx, faction, unit_id, form)?;
    set_behemoth_slayer(ctx, faction, slot, behemoth_slayer as u8)?;
    let behemoth_dodge_chance = stat_behemoth_dodge_chance(ctx, faction, unit_id, form)?;
    set_behemoth_dodge_chance(ctx, faction, slot, behemoth_dodge_chance)?;
    let behemoth_dodge_duration = stat_behemoth_dodge_duration(ctx, faction, unit_id, form)?;
    set_behemoth_dodge_duration(ctx, faction, slot, behemoth_dodge_duration)?;
    let sage_slayer = stat_sage_slayer(ctx, faction, unit_id, form)?;
    set_sage_slayer(ctx, faction, slot, sage_slayer as u8)?;
    let metal_killer_pct = stat_metal_killer_percent(ctx, faction, unit_id, form)?;
    set_metal_killer_pct(ctx, faction, slot, metal_killer_pct)?;
    let drain_chance = stat_drain_chance(ctx, faction, unit_id)?;
    set_drain_chance(ctx, faction, slot, drain_chance)?;
    let drain_percent = stat_drain_percent(ctx, faction, unit_id)?;
    set_drain_percent(ctx, faction, slot, drain_percent)?;
    let wave_block = stat_wave_block(ctx, faction, unit_id, form)?;
    set_wave_block(ctx, faction, slot, wave_block as i32)?;
    let trait_dojo = trait_dojo(ctx, faction, unit_id)?;
    set_trait_dojo(ctx, faction, slot, trait_dojo as i32)?;
    let boss_wave_immune = stat_boss_wave_immune(ctx, faction, unit_id, form)?;
    set_boss_wave_immune(ctx, faction, slot, boss_wave_immune as i32)?;
    let barrier_hp = stat_barrier_hitpoints(ctx, faction, unit_id)?;
    set_barrier_hp(ctx, faction, slot, barrier_hp)?;
    let shield_max = stat_shield_hitpoints(ctx, faction, unit_id, form, mag_slot)?;
    set_shield_max(ctx, faction, slot, shield_max)?;
    let shield_regen = stat_shield_regen(ctx, faction, unit_id)?;
    set_shield_regen(ctx, faction, slot, shield_regen)?;
    let warp_immune = stat_warp_immune(ctx, faction, unit_id, form)?;
    set_warp_immune(ctx, faction, slot, warp_immune as i32)?;
    let dodge_chance = stat_dodge_chance(ctx, faction, unit_id, form)?;
    set_dodge_chance(ctx, faction, slot, dodge_chance)?;
    let dodge_duration = stat_dodge_duration(ctx, faction, unit_id, form)?;
    set_dodge_duration(ctx, faction, slot, dodge_duration)?;
    set_orb_dodge_chance(ctx, faction, slot, 0)?;
    set_orb_dodge_duration(ctx, faction, slot, 0)?;

    if !low
        && orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)?
        && has_orb(ctx, &ctx.orb_store, unit_id, 0xd)?
    {
        let orb_dodge_chance = get_orb_value_max(ctx, unit_id, 0xd, 0, 0)?;
        set_orb_dodge_chance(ctx, 0, slot, orb_dodge_chance)?;
        let orb_dodge_duration = get_orb_value_max(ctx, unit_id, 0xd, 1, 0)?;
        set_orb_dodge_duration(ctx, 0, slot, orb_dodge_duration)?;
    }

    set_crit_vfx(ctx, faction, slot, 0)?;
    set_kb_proc_hit(ctx, faction, slot, 0)?;
    set_freeze_timer(ctx, faction, slot, 0)?;
    set_freeze_length(ctx, faction, slot, 0)?;
    set_prev_freeze_timer(ctx, faction, slot, 0)?;
    set_slow_timer(ctx, faction, slot, 0)?;
    set_slow_length(ctx, faction, slot, 0)?;
    set_prev_slow_timer(ctx, faction, slot, 0)?;
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::DOUBLE_BOUNTY_STATE),
        0,
    )?;
    set_shockwave_counter(ctx, faction, slot, 0)?;
    set_weaken_timer(ctx, faction, slot, 0)?;
    set_weaken_active(ctx, faction, slot, 0)?;
    set_prev_weaken_timer(ctx, faction, slot, 0)?;
    set_weaken_active_pct(ctx, faction, slot, 0)?;
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::SURVIVE_USED),
        0,
    )?;
    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::ATTACK_UP_VFX_FRAME),
        0,
    )?;
    set_savage_blow_vfx(ctx, faction, slot, 0)?;
    set_dodge_timer(ctx, faction, slot, 0)?;
    set_toxic_vfx(ctx, faction, slot, 0)?;
    let shield_hp = stat_shield_hitpoints(ctx, faction, unit_id, form, mag_slot)?;
    set_shield_hp(ctx, faction, slot, shield_hp)?;
    set_shield_vfx(ctx, faction, slot, 0)?;
    set_behemoth_dodge_timer(ctx, faction, slot, 0)?;
    set_metal_killer_vfx(ctx, faction, slot, 0)?;
    set_orb_dodge_timer(ctx, faction, slot, 0)?;
    set_proc_badge(ctx, faction, slot, 0, 0)?;
    set_proc_badge(ctx, faction, slot, 1, 0)?;
    set_proc_badge(ctx, faction, slot, 2, 0)?;
    set_proc_badge(ctx, faction, slot, 3, 0)?;
    set_proc_badge(ctx, faction, slot, 4, 0)?;
    ctx.zero(
        AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_VFX_FRAME),
        0x14,
    )?;
    set_barrier_vfx_active(ctx, faction, slot, 0)?;
    set_warp_timer(ctx, faction, slot, 0)?;
    set_curse_timer(ctx, faction, slot, 0)?;
    set_curse_length(ctx, faction, slot, 0)?;
    set_prev_curse_timer(ctx, faction, slot, 0)?;

    let mut weaken = 0x64i32;

    if !stat_weaken_immune(ctx, faction, unit_id, form)? {
        weaken = get_talent_value(ctx, faction, unit_id, form, 0x12, 0)?;

        if !low {
            weaken = get_orb_value_sum(ctx, unit_id, 0x15, 0, 0)?.wrapping_add(weaken);
        }
    }

    set_weaken_immune(ctx, faction, slot, 0)?;
    set_weaken_resist_pct(ctx, faction, slot, 0)?;

    if weaken < 0x64 {
        set_weaken_resist_pct(ctx, faction, slot, weaken)?;
    } else {
        set_weaken_immune(ctx, faction, slot, 1)?;
    }

    let mut freeze = 0x64i32;

    if !stat_freeze_immune(ctx, faction, unit_id, form)? {
        freeze = get_talent_value(ctx, faction, unit_id, form, 0x13, 0)?;

        if !low {
            freeze = get_orb_value_sum(ctx, unit_id, 0x14, 0, 0)?.wrapping_add(freeze);
        }
    }

    set_freeze_immune(ctx, faction, slot, 0)?;
    set_freeze_resist_pct(ctx, faction, slot, 0)?;

    if freeze < 0x64 {
        set_freeze_resist_pct(ctx, faction, slot, freeze)?;
    } else {
        set_freeze_immune(ctx, faction, slot, 1)?;
    }

    let mut slow = 0x64i32;

    if !stat_slow_immune(ctx, faction, unit_id, form)? {
        slow = get_talent_value(ctx, faction, unit_id, form, 0x14, 0)?;

        if !low {
            slow = get_orb_value_sum(ctx, unit_id, 0xe, 0, 0)?.wrapping_add(slow);
        }
    }

    set_slow_immune(ctx, faction, slot, 0)?;
    set_slow_resist_pct(ctx, faction, slot, 0)?;

    if slow < 0x64 {
        set_slow_resist_pct(ctx, faction, slot, slow)?;
    } else {
        set_slow_immune(ctx, faction, slot, 1)?;
    }

    let mut knockback = 0x64i32;

    if !stat_knockback_immune(ctx, faction, unit_id, form)? {
        knockback = get_talent_value(ctx, faction, unit_id, form, 0x15, 0)?;

        if !low {
            knockback = get_orb_value_sum(ctx, unit_id, 0x8, 0, 0)?.wrapping_add(knockback);
        }
    }

    set_knockback_immune(ctx, faction, slot, 0)?;
    set_knockback_resist_pct(ctx, faction, slot, 0)?;

    if knockback < 0x64 {
        set_knockback_resist_pct(ctx, faction, slot, knockback)?;
    } else {
        set_knockback_immune(ctx, faction, slot, 1)?;
    }

    let mut wave = 0x64i32;

    if !stat_wave_immune(ctx, faction, unit_id, form)? {
        wave = get_talent_value(ctx, faction, unit_id, form, 0x16, 0)?;

        if !low {
            wave = get_orb_value_sum(ctx, unit_id, 0x6, 0, 0)?.wrapping_add(wave);
        }
    }

    if faction == 0 && get_cat_combo_bonus(ctx, &ctx.combo_store, 0x1a, unit_id)? > 0 {
        wave = wave.wrapping_add(0x64);
    }

    set_wave_immune(ctx, faction, slot, 0)?;
    set_wave_resist_pct(ctx, faction, slot, 0)?;

    if wave < 0x64 {
        set_wave_resist_pct(ctx, faction, slot, wave)?;
    } else {
        set_wave_immune(ctx, faction, slot, 1)?;
    }

    let warp_resist_pct = get_talent_value(ctx, faction, unit_id, form, 0x18, 0)?;
    set_warp_resist_pct(ctx, faction, slot, warp_resist_pct)?;

    let mut curse = 0x64i32;

    if !stat_curse_immune(ctx, faction, unit_id, form)? {
        curse = get_talent_value(ctx, faction, unit_id, form, 0x1e, 0)?;

        if !low {
            curse = get_orb_value_sum(ctx, unit_id, 0xf, 0, 0)?.wrapping_add(curse);
        }
    }

    set_curse_immune(ctx, faction, slot, 0)?;
    set_curse_resist_pct(ctx, faction, slot, 0)?;

    if curse < 0x64 {
        set_curse_resist_pct(ctx, faction, slot, curse)?;
    } else {
        set_curse_immune(ctx, faction, slot, 1)?;
    }

    let mut toxic = 0x64i32;

    if !stat_toxic_immune(ctx, faction, unit_id, form)? {
        toxic = get_talent_value(ctx, faction, unit_id, form, 0x34, 0)?;

        if !low {
            toxic = get_orb_value_sum(ctx, unit_id, 0xc, 0, 0)?.wrapping_add(toxic);
        }
    }

    set_toxic_immune(ctx, faction, slot, 0)?;
    set_toxic_resist_pct(ctx, faction, slot, 0)?;

    if toxic < 0x64 {
        set_toxic_resist_pct(ctx, faction, slot, toxic)?;
    } else {
        set_toxic_immune(ctx, faction, slot, 1)?;
    }

    let mut surge = 0x64i32;

    if !stat_surge_immune(ctx, faction, unit_id, form)? {
        surge = get_talent_value(ctx, faction, unit_id, form, 0x36, 0)?;

        if !low {
            surge = get_orb_value_sum(ctx, unit_id, 0x17, 0, 0)?.wrapping_add(surge);
        }
    }

    if faction == 0 && get_cat_combo_bonus(ctx, &ctx.combo_store, 0x1c, unit_id)? > 0 {
        surge = surge.wrapping_add(0x64);
    }

    set_surge_immune(ctx, faction, slot, 0)?;
    set_surge_resist_pct(ctx, faction, slot, 0)?;

    if surge < 0x64 {
        set_surge_resist_pct(ctx, faction, slot, surge)?;
    } else {
        set_surge_immune(ctx, faction, slot, 1)?;
    }

    let explosion_immune = stat_explosion_immune(ctx, faction, unit_id, form)?;
    let mut explosion = if explosion_immune { 0x64i32 } else { 0 };

    if !low && !explosion_immune {
        explosion = get_orb_value_sum(ctx, unit_id, 0x19, 0, 0)?;
    }

    set_explosion_immune(ctx, faction, slot, 0)?;
    set_explosion_resist_pct(ctx, faction, slot, 0)?;

    if explosion < 0x64 {
        set_explosion_resist_pct(ctx, faction, slot, explosion)?;
    } else {
        set_explosion_immune(ctx, faction, slot, 1)?;
    }

    let drain_immune = stat_drain_immune(ctx, faction, unit_id, form)?;

    set_drain_immune(ctx, faction, slot, 0)?;

    if drain_immune {
        set_drain_immune(ctx, faction, slot, 1)?;
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        let enemy_row =
            *ctx.stage_enemies
                .get(mag_slot as i64 as usize)
                .ok_or(Fault::index_out_of_range(mag_slot as i64, ctx.stage_enemies.len() as i64))?;

        set_boss_type(ctx, faction, slot, stage_entry_boss(&enemy_row))?;

        if is_boss(ctx, faction, slot)? {
            let base_x = get_base_pos_x(ctx, 1)?;
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let inset = ops::div_neg_100(size.wrapping_mul(0x49c) as i64) as i32;
            let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let shift = ops::div_10(size.wrapping_mul(offset_x) as i64) as i32;
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let width = ops::div_10((size << 7).wrapping_sub(size) as i64) as i32;

            set_pos_x(
                ctx,
                faction,
                slot,
                inset
                    .wrapping_add(base_x)
                    .wrapping_add(width)
                    .wrapping_add(shift),
            )?;
        }

        let stats = (row as i64) * ENEMY_STATS_STRIDE as i64 + ENEMY_STATS as i64;

        set_burrow_count(
            ctx,
            faction,
            slot,
            ctx.i32_at((stats + EnemyStats::BURROW_AMOUNT as i64) as usize)?,
        )?;
        set_revive_count(
            ctx,
            faction,
            slot,
            ctx.i32_at((stats + EnemyStats::REVIVE_COUNT as i64) as usize)?,
        )?;
        set_revive_timer(ctx, faction, slot, 0)?;
        set_revive_hp(
            ctx,
            faction,
            slot,
            ctx.i32_at((stats + EnemyStats::REVIVE_HP as i64) as usize)?,
        )?;
        set_revive_time(
            ctx,
            faction,
            slot,
            ctx.i32_at((stats + EnemyStats::REVIVE_TIME as i64) as usize)?,
        )?;
        set_burrow_distance(
            ctx,
            faction,
            slot,
            ctx.i32_at((stats + EnemyStats::BURROW_DISTANCE as i64) as usize)?,
        )?;
        set_zkill_hit(ctx, faction, slot, 0)?;
        set_no_revive(ctx, faction, slot, 0)?;

        let enemy_row =
            *ctx.stage_enemies
                .get(mag_slot as i64 as usize)
                .ok_or(Fault::index_out_of_range(mag_slot as i64, ctx.stage_enemies.len() as i64))?;

        set_score_value(ctx, faction, slot, stage_entry_col10(&enemy_row))?;
    } else {
        set_boss_type(ctx, faction, slot, 0)?;
    }

    let mut first_bounty = 0i32;

    if !low {
        let mut cannon_charge = 0i32;

        if orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)? {
            cannon_charge = get_orb_value_max(ctx, unit_id, 0xb, 0, 0)?;
        }

        set_cannon_charge_orb(ctx, 0, slot, cannon_charge)?;
        set_paid_cost(ctx, 0, slot, 0)?;

        if orb_deploy_condition_slot(ctx, AppContext::faction_flags(faction), 0, slot)? {
            first_bounty = get_orb_value_max(ctx, unit_id, 0x18, 0, 0)?;
        }
    } else {
        set_cannon_charge_orb(ctx, faction, slot, 0)?;
        set_paid_cost(ctx, faction, slot, 0)?;
    }

    set_first_bounty_pct(ctx, faction, slot, first_bounty)?;

    let spawn_serial =
        AppContext::faction_flags(faction).wrapping_add(AppContext::WALLET_SPAWN_SERIAL);
    let serial = ctx.i32_at(spawn_serial)?;

    ctx.set_i32_at(spawn_serial, serial.wrapping_add(1))?;
    set_spawn_serial(ctx, faction, slot, serial)?;
    set_kill_count(ctx, faction, slot, 0)?;

    let mut state = 0xa;

    if !get_spawn_anim_flag(ctx, faction, slot)? && get_spawn_anim_type(ctx, faction, slot)? < 0 {
        state = 0;
    }

    set_entity_state(ctx, faction, slot, state)?;

    Ok(slot)
}
