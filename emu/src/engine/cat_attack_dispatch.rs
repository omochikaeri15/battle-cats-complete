use crate::{operation, Fault};

use super::{
    add_score_hit_mask, attack_dmg_dispatch, attack_proc_dispatch, battle_not_finishing, build_trait_mask, call_rng, compute_attack, count_target_traits,
    does_target, get_attack_damage, get_barrier_hp, get_base_destroyer, get_best_treasure, get_button_unit_form, get_cat_combo_bonus,
    get_colossus_orb_atk_pct, get_dodge_chance, get_dodge_duration, get_dodge_timer, get_entity_base_idx, get_entity_button, get_explosion_immune, get_hp,
    get_max_hp, get_orb_value_max, get_orb_value_vs_trait, get_savage_blow_boost, get_score_bonus, get_setting, get_shield_hp, get_slot_unit_id,
    get_spawn_serial, get_status_bits, get_strengthen_boost, get_strengthen_threshold, get_surge_immune, get_trait_kaijin, get_treasure_capped,
    get_wave_immune, get_weaken_active, get_weaken_active_pct, get_weaken_timer, has_behemoth_slayer, has_colossus_slayer, has_double_bounty,
    has_eva_killer, has_insane_damage, has_massive_damage, has_sage_slayer, has_strong_against, has_witch_slayer, has_zombie_killer, is_alien, is_aku,
    is_angel, is_behemoth, is_colossus, is_dark, is_eva_angel, is_floating, is_metal, is_red, is_relic, is_sage, is_touchable_thunk, is_traitless,
    is_witch, is_zombie, set_barrier_state, set_crit_fx, set_dodge_fx_frame, set_dodge_timer, set_metal_killer_fx, set_savage_blow_fx, set_shield_state,
    set_zkill_hit, AppContext, Entity,
};

#[allow(clippy::too_many_arguments)]
pub fn cat_attack_dispatch(
    ctx: &mut AppContext,
    source: i32,
    attacker: i32,
    target: i32,
    attack: i32,
    mode: i32,
    crit: u8,
    savage: u8,
    p_knockback: u8,
    p_freeze: u8,
    p_slow: u8,
    p_weaken: u8,
    p_warp: u8,
    p_curse: u8,
    barrier_broke: u8,
    shield_pierced: u8,
    metal_killer_pct: i32,
    dmg_scale: i32,
) -> Result<bool, Fault> {
    if !is_touchable_thunk(ctx, 1, target, attacker)? {
        return Ok(false);
    }

    let traits = build_trait_mask(&[
        (0x1, is_red(ctx, 1, target)?),
        (0x2, is_floating(ctx, 1, target)?),
        (0x4, is_dark(ctx, 1, target)?),
        (0x8, is_metal(ctx, 1, target)?),
        (0x10, is_angel(ctx, 1, target)?),
        (0x20, is_alien(ctx, 1, target)?),
        (0x40, is_zombie(ctx, 1, target)?),
        (0x80, is_relic(ctx, 1, target)?),
        (0x100, is_traitless(ctx, 1, target)?),
        (0x200, is_witch(ctx, 1, target)?),
        (0x400, is_eva_angel(ctx, 1, target)?),
        (0x800, is_aku(ctx, 1, target)?),
    ]);
    let unit_id = get_slot_unit_id(ctx, 0, attacker)?;
    let button = get_entity_button(ctx, 0, attacker)?;
    let form = get_button_unit_form(ctx, 0, button)?;
    let raw = get_attack_damage(ctx, 0, attacker, attack)?.wrapping_mul(dmg_scale);
    let mut damage: i64;

    if mode == 2 {
        damage = operation::div_500(raw) as i64;
    } else {
        damage = operation::div_100(raw) as i64;

        if mode == 1 {
            let percent = get_setting(&ctx.settings, b"battle_wave_s_st", 0x1e)? as i64;

            damage = operation::div_100(percent.wrapping_mul(damage));
        }
    }

    if crit != 0 {
        set_crit_fx(ctx, 1, target, 1)?;
        damage = damage.wrapping_add(damage);
    }

    let hp = get_hp(ctx, 0, attacker)?;
    let max_hp = get_max_hp(ctx, 0, attacker)?;

    if hp <= operation::div_100(get_strengthen_threshold(ctx, 0, attacker)?.wrapping_mul(max_hp)) {
        let max_hp = get_max_hp(ctx, 0, attacker)?;

        if get_strengthen_threshold(ctx, 0, attacker)?.wrapping_mul(max_hp) >= 100 {
            let boost = get_strengthen_boost(ctx, 0, attacker)?;
            let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x15, unit_id)?;

            get_strengthen_boost(ctx, 0, attacker)?;
            damage = damage.wrapping_mul(combo.wrapping_add(boost).wrapping_add(100) as i64);
            get_cat_combo_bonus(ctx, &ctx.combo_store, 0x15, unit_id)?;
            damage = operation::div_100(damage);
        }
    }

    if get_status_bits(ctx, 0, attacker)? & 2 != 0 {
        let percent = get_orb_value_max(ctx, unit_id, 0x12, 0, 0)?.wrapping_add(100) as i64;

        damage = operation::div_100(damage.wrapping_mul(percent));
    }

    if get_weaken_timer(ctx, 0, attacker)? > 0 && get_weaken_active(ctx, 0, attacker)? > 0 {
        let percent = get_weaken_active_pct(ctx, 0, attacker)? as i64;

        damage = operation::div_100(damage.wrapping_mul(percent));
    }

    if savage != 0 {
        let boost = get_savage_blow_boost(ctx, 0, attacker)?;

        get_savage_blow_boost(ctx, 0, attacker)?;
        damage = damage.wrapping_mul(boost.wrapping_add(100) as i64);
        set_savage_blow_fx(ctx, 1, target, 1)?;
        damage = operation::div_100(damage);
    }

    if target == 0 || get_entity_base_idx(ctx)? == target {
        if get_base_destroyer(ctx, 0, attacker)? {
            damage = damage.wrapping_mul(4);
        }
    } else {
        if does_target(ctx, 1, target, attacker)? {
            if get_dodge_timer(ctx, 1, target)? > 0 {
                return Ok(false);
            }

            let roll = call_rng(ctx, 100);

            if roll < get_dodge_chance(ctx, 1, target)? {
                let duration = get_dodge_duration(ctx, 1, target)?;

                set_dodge_timer(ctx, 1, target, duration)?;
                set_dodge_fx_frame(ctx, 1, target, 1)?;
                get_dodge_chance(ctx, 1, target)?;
                get_dodge_chance(ctx, 1, target)?;

                return Ok(false);
            }
        }

        if does_target(ctx, 0, attacker, target)? {
            let tier = get_best_treasure(ctx, attacker, target)?;

            if has_strong_against(ctx, 0, attacker)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;
                let mut orb = 0;

                if form >= 2 {
                    orb = get_orb_value_vs_trait(ctx, unit_id, 2, 0, &traits, 0)?;
                }

                let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xe, unit_id)?;

                damage = operation::div_1000(damage.wrapping_mul(treasure.wrapping_add(orb).wrapping_add(0x5dc) as i64));
                damage = (combo.wrapping_add(100) as i64).wrapping_mul(damage);
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0xe, unit_id)?;

                let scoring = battle_not_finishing(ctx)?;

                damage = operation::div_100(damage);

                if source == 0 && scoring {
                    let traits_hit = count_target_traits(ctx, 0, attacker)?;
                    let bonus = get_score_bonus(ctx, 0xd, traits_hit)?;

                    add_score_hit_mask(ctx, 1, target, bonus)?;
                }
            }

            if has_massive_damage(ctx, 0, attacker)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;
                let mut orb = 0;

                if form >= 2 {
                    orb = get_orb_value_vs_trait(ctx, unit_id, 3, 0, &traits, 0)?;
                }

                let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xf, unit_id)?;

                damage = operation::div_300(damage.wrapping_mul(treasure.wrapping_add(orb).wrapping_add(0x384) as i64));
                damage = (combo.wrapping_add(100) as i64).wrapping_mul(damage);
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0xf, unit_id)?;

                let scoring = battle_not_finishing(ctx)?;

                damage = operation::div_100(damage);

                if source == 0 && scoring {
                    let traits_hit = count_target_traits(ctx, 0, attacker)?;
                    let bonus = get_score_bonus(ctx, 0xe, traits_hit)?;

                    add_score_hit_mask(ctx, 1, target, bonus)?;
                }
            }

            if has_insane_damage(ctx, 0, attacker)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;

                damage = damage.wrapping_mul(treasure.wrapping_add(0x5dc) as i64);

                let scoring = battle_not_finishing(ctx)?;

                damage = operation::div_300(damage);

                if source == 0 && scoring {
                    let traits_hit = count_target_traits(ctx, 0, attacker)?;
                    let bonus = get_score_bonus(ctx, 0xf, traits_hit)?;

                    add_score_hit_mask(ctx, 1, target, bonus)?;
                }
            }

            let immune = match source {
                3 => get_explosion_immune(ctx, 1, target)?,
                2 => get_surge_immune(ctx, 1, target)?,
                1 => get_wave_immune(ctx, 1, target)?,
                _ => false,
            };

            if !immune {
                attack_proc_dispatch(
                    ctx,
                    0,
                    attacker,
                    target,
                    tier,
                    p_knockback as i32,
                    p_freeze as i32,
                    p_slow as i32,
                    p_weaken as i32,
                    p_warp,
                    p_curse,
                    0,
                )?;
            }
        }

        if has_witch_slayer(ctx, 0, attacker)? && is_witch(ctx, 1, target)? {
            let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x16, unit_id)?;

            damage = damage.wrapping_mul(combo.wrapping_add(100) as i64).wrapping_mul(5);
            get_cat_combo_bonus(ctx, &ctx.combo_store, 0x16, unit_id)?;
            damage = operation::div_100(damage);
        }

        if has_eva_killer(ctx, 0, attacker)? && is_eva_angel(ctx, 1, target)? {
            let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x17, unit_id)?;

            damage = damage.wrapping_mul(combo.wrapping_add(100) as i64).wrapping_mul(5);
            get_cat_combo_bonus(ctx, &ctx.combo_store, 0x17, unit_id)?;
            damage = operation::div_100(damage);
        }

        if has_colossus_slayer(ctx, 0, attacker)? && is_colossus(ctx, 1, target)? {
            let percent = get_colossus_orb_atk_pct(ctx, 0, attacker)? as i64;

            damage = damage.wrapping_mul(percent);
            get_colossus_orb_atk_pct(ctx, 0, attacker)?;
            damage = operation::div_100(damage);
        }

        if has_behemoth_slayer(ctx, 0, attacker)? && is_behemoth(ctx, 1, target)? {
            let permille = get_setting(&ctx.settings, b"battle_super_beast_hunter", 0x640)? as i64;

            damage = damage.wrapping_mul(permille);
            get_setting(&ctx.settings, b"battle_super_beast_hunter", 0x640)?;
            damage = operation::div_1000(damage);
        }

        if has_sage_slayer(ctx, 0, attacker)? && is_sage(ctx, 1, target)? {
            let permille = get_setting(&ctx.settings, b"battle_super_sage_hunter_damage1", 0x640)? as i64;

            damage = damage.wrapping_mul(permille);
            get_setting(&ctx.settings, b"battle_super_sage_hunter_damage1", 0x640)?;
            damage = operation::div_1000(damage);
        }

        let kaijin_combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x19, unit_id)?;

        if kaijin_combo > 0 && get_trait_kaijin(ctx, 1, target)? {
            damage = operation::div_1000(damage.wrapping_mul(kaijin_combo as u32 as i64));
        }

        if get_barrier_hp(ctx, 1, target)? > 0 && barrier_broke != 0 {
            set_barrier_state(ctx, 1, target, 2)?;
        }

        if get_shield_hp(ctx, 1, target)? > 0 && shield_pierced != 0 {
            set_shield_state(ctx, 1, target, 2)?;
        }
    }

    if form >= 2 {
        let percent = get_orb_value_vs_trait(ctx, unit_id, 0, 0, &traits, 0)?;

        if percent > 0 {
            let base = compute_attack(ctx, 0, unit_id, form, 1, -1, attack, 1, 0)?;

            compute_attack(ctx, 0, unit_id, form, 1, -1, attack, 1, 0)?;
            damage = damage.wrapping_add(operation::div_100(base.wrapping_mul(percent)) as i64);
        }
    }

    let metal = is_metal(ctx, 1, target)?;
    let mut capped = 1i64;

    if damage <= 0 {
        capped = damage;
    }

    if crit != 0 {
        capped = damage;
    }

    if !metal {
        capped = damage;
    }

    let mut dealt = if capped > 0 { capped } else { 0 };

    let immune = match source {
        3 => get_explosion_immune(ctx, 1, target)?,
        2 => get_surge_immune(ctx, 1, target)?,
        1 => get_wave_immune(ctx, 1, target)?,
        _ => false,
    };

    if immune {
        ctx.set_i32_at(AppContext::entity_field(1, target, Entity::WAVE_IMMUNE_FX_FRAME), 0)?;
        ctx.set_i32_at(AppContext::entity_field(1, target, Entity::WAVE_IMMUNE_FX_ACTIVE), 1)?;
        dealt = 0;
    } else if is_metal(ctx, 1, target)? && metal_killer_pct > 0 {
        ctx.metal_killer_map.entry(target).or_default().push(metal_killer_pct);
        set_metal_killer_fx(ctx, 1, target, 1)?;
    }

    let attacker_serial = get_spawn_serial(ctx, 0, attacker)?;
    let victim_serial = get_spawn_serial(ctx, 1, target)?;

    if !ctx.attackers_by_serial[1].entry(victim_serial).or_default().contains(&attacker_serial) {
        ctx.attackers_by_serial[1].entry(victim_serial).or_default().push(attacker_serial);
    }

    attack_dmg_dispatch(ctx, 1, target, dealt as i32)?;

    if has_double_bounty(ctx, 0, attacker)? {
        ctx.set_i32_at(AppContext::entity_field(1, target, Entity::DOUBLE_BOUNTY_STATE), 1)?;
    }

    if has_zombie_killer(ctx, 0, attacker)? && is_zombie(ctx, 1, target)? {
        set_zkill_hit(ctx, 1, target, 1)?;
    }

    if battle_not_finishing(ctx)? && dealt != 0 && source.wrapping_sub(1) as u32 <= 2 {
        let bonus = get_score_bonus(ctx, source.wrapping_add(9), 1)?;

        add_score_hit_mask(ctx, 1, target, bonus)?;
    }

    Ok(true)
}
