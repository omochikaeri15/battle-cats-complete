use crate::{Fault, ops};

use super::{
    AppContext, Entity, add_score_hit_mask, attack_dmg_dispatch, attack_proc_dispatch,
    battle_not_finishing, build_trait_mask, call_rng, count_target_traits, does_target,
    get_attack_damage, get_barrier_hp, get_base_destroyer, get_behemoth_dodge_chance,
    get_behemoth_dodge_duration, get_behemoth_dodge_timer, get_best_treasure, get_button_unit_form,
    get_cannon_effect, get_cat_combo_bonus, get_colossus_orb_def_pct, get_dodge_chance,
    get_dodge_duration, get_dodge_timer, get_entity_button, get_explosion_immune,
    get_explosion_resist_pct, get_foundation_part_id, get_foundation_part_level,
    get_foundation_part_rec, get_hp, get_max_hp, get_orb_dodge_chance, get_orb_dodge_duration,
    get_orb_dodge_timer, get_orb_value_vs_trait, get_resist_part_rec, get_savage_blow_boost,
    get_score_bonus, get_setting, get_shield_hp, get_slot_unit_id, get_strengthen_boost,
    get_strengthen_threshold, get_style_part_id, get_style_part_level, get_surge_immune,
    get_surge_resist_pct, get_toxic_damage, get_toxic_immune, get_toxic_resist_pct,
    get_trait_kaijin, get_treasure_capped, get_wave_immune, get_wave_resist_pct, get_weaken_active,
    get_weaken_active_pct, get_weaken_timer, has_behemoth_slayer, has_colossus_slayer,
    has_conjure_deck_slot, has_eva_killer, has_fixed_lineup, has_insanely_tough, has_resist,
    has_sage_slayer, has_strong_against, has_witch_slayer, is_aku, is_alien, is_angel, is_behemoth,
    is_colossus, is_dark, is_eva_angel, is_floating, is_metal, is_red, is_relic, is_sage,
    is_touchable_thunk, is_traitless, is_witch, is_zombie, max_i32, set_barrier_state,
    set_behemoth_dodge_timer, set_crit_vfx, set_dodge_timer, set_dodge_vfx_frame,
    set_orb_dodge_timer, set_savage_blow_vfx, set_shield_state, set_toxic_vfx, start_immune_vfx,
};

pub fn enemy_attack_dispatch(
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
    p_toxic: u8,
    shield_pierced: u8,
    p_drain: u8,
    dmg_scale: i32,
) -> Result<bool, Fault> {
    let button = get_entity_button(ctx, 0, target)?;

    get_button_unit_form(ctx, 0, button)?;

    if !is_touchable_thunk(ctx, 0, target, attacker)? {
        return Ok(false);
    }

    let traits = build_trait_mask(&[
        (0x1, is_red(ctx, 1, attacker)?),
        (0x2, is_floating(ctx, 1, attacker)?),
        (0x4, is_dark(ctx, 1, attacker)?),
        (0x8, is_metal(ctx, 1, attacker)?),
        (0x10, is_angel(ctx, 1, attacker)?),
        (0x20, is_alien(ctx, 1, attacker)?),
        (0x40, is_zombie(ctx, 1, attacker)?),
        (0x80, is_relic(ctx, 1, attacker)?),
        (0x100, is_traitless(ctx, 1, attacker)?),
        (0x200, is_witch(ctx, 1, attacker)?),
        (0x400, is_eva_angel(ctx, 1, attacker)?),
        (0x800, is_aku(ctx, 1, attacker)?),
    ]);
    let unit_id = get_slot_unit_id(ctx, 0, target)?;
    let button = get_entity_button(ctx, 0, target)?;
    let form = get_button_unit_form(ctx, 0, button)?;
    let raw = get_attack_damage(ctx, 1, attacker, attack)?.wrapping_mul(dmg_scale);
    let mut damage: i64;

    if mode == 2 {
        damage = ops::div_500(raw) as i64;
    } else {
        damage = ops::div_100(raw) as i64;

        if mode == 1 {
            let percent = get_setting(&ctx.settings, b"battle_wave_s_st", 0x1e)? as i64;

            damage = ops::div_100(percent.wrapping_mul(damage));
        }
    }

    if crit != 0 {
        set_crit_vfx(ctx, 0, target, 1)?;
        damage = damage.wrapping_add(damage);
    }

    let hp = get_hp(ctx, 1, attacker)?;
    let max_hp = get_max_hp(ctx, 1, attacker)?;

    if hp <= ops::div_100(get_strengthen_threshold(ctx, 1, attacker)?.wrapping_mul(max_hp)) {
        let max_hp = get_max_hp(ctx, 1, attacker)?;

        if get_strengthen_threshold(ctx, 1, attacker)?.wrapping_mul(max_hp) >= 100 {
            let boost = get_strengthen_boost(ctx, 1, attacker)?;

            damage = damage.wrapping_mul(boost.wrapping_add(100) as i64);
            get_strengthen_boost(ctx, 1, attacker)?;
            damage = ops::div_100(damage);
        }
    }

    if get_weaken_timer(ctx, 1, attacker)? > 0 && get_weaken_active(ctx, 1, attacker)? > 0 {
        let percent = get_weaken_active_pct(ctx, 1, attacker)? as i64;

        damage = ops::div_100(damage.wrapping_mul(percent));
    }

    if savage != 0 {
        let boost = get_savage_blow_boost(ctx, 1, attacker)?;

        get_savage_blow_boost(ctx, 1, attacker)?;
        damage = damage.wrapping_mul(boost.wrapping_add(100) as i64);
        set_savage_blow_vfx(ctx, 0, target, 1)?;
        damage = ops::div_100(damage);
    }

    if target == 0 {
        if get_base_destroyer(ctx, 1, attacker)? {
            damage = damage.wrapping_mul(4);
        }
    } else {
        let mut lands = true;

        if is_behemoth(ctx, 1, attacker)? {
            if get_behemoth_dodge_timer(ctx, 0, target)? > 0 {
                return Ok(false);
            }

            let roll = call_rng(ctx, 100);

            if roll < get_behemoth_dodge_chance(ctx, 0, target)? {
                let duration = get_behemoth_dodge_duration(ctx, 0, target)?;

                set_behemoth_dodge_timer(ctx, 0, target, duration)?;
                set_dodge_vfx_frame(ctx, 0, target, 1)?;
                get_behemoth_dodge_chance(ctx, 0, target)?;
                lands = false;
                get_behemoth_dodge_chance(ctx, 0, target)?;
            }
        }

        if get_orb_dodge_timer(ctx, 0, target)? > 0 {
            return Ok(false);
        }

        if does_target(ctx, 0, target, attacker)? || has_conjure_deck_slot(ctx, 0, target)? {
            if get_dodge_timer(ctx, 0, target)? > 0 {
                return Ok(false);
            }

            let roll = call_rng(ctx, 100);

            if roll < get_dodge_chance(ctx, 0, target)? {
                let tier = get_best_treasure(ctx, target, attacker)?;
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;
                let duration = ops::div_1500(
                    get_dodge_duration(ctx, 0, target)?.wrapping_mul(treasure.wrapping_add(0x5dc)),
                );

                set_dodge_timer(ctx, 0, target, duration)?;
                set_dodge_vfx_frame(ctx, 0, target, 1)?;
                get_dodge_chance(ctx, 0, target)?;
                get_dodge_chance(ctx, 0, target)?;

                return Ok(false);
            }

            if !lands {
                return Ok(false);
            }

            let tier = get_best_treasure(ctx, target, attacker)?;

            if has_strong_against(ctx, 0, target)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;
                let mut orb = 0;

                if form >= 2 {
                    orb = get_orb_value_vs_trait(ctx, unit_id, 2, 1, &traits, 0)?;
                }

                let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xe, unit_id)?;

                damage = ops::div_3000(
                    damage.wrapping_mul(0x5dci32.wrapping_sub(treasure) as i64),
                );
                damage =
                    ops::div_100((100i32.wrapping_sub(combo) as i64).wrapping_mul(damage));
                damage = (100i32.wrapping_sub(orb) as i64).wrapping_mul(damage);
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0xe, unit_id)?;

                let scoring = battle_not_finishing(ctx)?;

                damage = ops::div_100(damage);

                if scoring {
                    let traits_hit = count_target_traits(ctx, 0, target)?;
                    let bonus = get_score_bonus(ctx, 0x10, traits_hit)?;

                    add_score_hit_mask(ctx, 0, target, bonus)?;
                }
            }

            if has_resist(ctx, 0, target)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;
                let mut orb = 0;

                if form >= 2 {
                    orb = get_orb_value_vs_trait(ctx, unit_id, 4, 0, &traits, 0)?;
                }

                let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x10, unit_id)?;

                damage = ops::div_6000(
                    damage.wrapping_mul(0x5dci32.wrapping_sub(treasure) as i64),
                );
                damage =
                    ops::div_100((100i32.wrapping_sub(combo) as i64).wrapping_mul(damage));
                damage = (100i32.wrapping_sub(orb) as i64).wrapping_mul(damage);
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0x10, unit_id)?;

                let scoring = battle_not_finishing(ctx)?;

                damage = ops::div_100(damage);

                if scoring {
                    let traits_hit = count_target_traits(ctx, 0, target)?;
                    let bonus = get_score_bonus(ctx, 0x11, traits_hit)?;

                    add_score_hit_mask(ctx, 0, target, bonus)?;
                }
            }

            if has_insanely_tough(ctx, 0, target)? {
                let treasure = get_treasure_capped(ctx, &ctx.treasure_store, tier, 0x64)?;

                damage = damage.wrapping_mul(0x834i32.wrapping_sub(treasure) as i64);

                let scoring = battle_not_finishing(ctx)?;

                damage = ops::div_12600(damage);

                if scoring {
                    let traits_hit = count_target_traits(ctx, 0, target)?;
                    let bonus = get_score_bonus(ctx, 0x12, traits_hit)?;

                    add_score_hit_mask(ctx, 0, target, bonus)?;
                }
            }
        } else if !lands {
            return Ok(false);
        }

        if get_orb_dodge_chance(ctx, 0, target)? > 0 {
            let roll = call_rng(ctx, 100);

            if roll < get_orb_dodge_chance(ctx, 0, target)? {
                let duration = get_orb_dodge_duration(ctx, 0, target)?;

                set_orb_dodge_timer(ctx, 0, target, duration)?;
                set_dodge_vfx_frame(ctx, 0, target, 1)?;
                get_orb_dodge_chance(ctx, 0, target)?;

                return Ok(false);
            }
        }

        if does_target(ctx, 1, attacker, target)? {
            let immune = match source {
                1 => get_wave_immune(ctx, 0, target)?,
                3 => get_explosion_immune(ctx, 0, target)?,
                2 => get_surge_immune(ctx, 0, target)?,
                _ => false,
            };

            if !immune {
                attack_proc_dispatch(
                    ctx,
                    1,
                    attacker,
                    target,
                    0x19,
                    p_knockback as i32,
                    p_freeze as i32,
                    p_slow as i32,
                    p_weaken as i32,
                    p_warp,
                    p_curse,
                    p_drain,
                )?;
            }
        }

        if has_witch_slayer(ctx, 0, target)? && is_witch(ctx, 1, attacker)? {
            let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x16, unit_id)?;

            damage = ops::div_10(damage)
                .wrapping_mul(ops::div_neg_100(combo).wrapping_add(5) as i64);
            get_cat_combo_bonus(ctx, &ctx.combo_store, 0x16, unit_id)?;
            damage = ops::div_5(damage);
        }

        if has_eva_killer(ctx, 0, target)? && is_eva_angel(ctx, 1, attacker)? {
            let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x17, unit_id)?;

            damage = ops::div_5(damage)
                .wrapping_mul(ops::div_neg_100(combo).wrapping_add(5) as i64);
            get_cat_combo_bonus(ctx, &ctx.combo_store, 0x17, unit_id)?;
            damage = ops::div_5(damage);
        }

        if has_colossus_slayer(ctx, 0, target)? && is_colossus(ctx, 1, attacker)? {
            let percent = get_colossus_orb_def_pct(ctx, 0, target)? as i64;

            damage = damage.wrapping_mul(percent);
            get_colossus_orb_def_pct(ctx, 0, target)?;
            damage = ops::div_100(damage);
        }

        if has_behemoth_slayer(ctx, 0, target)? && is_behemoth(ctx, 1, attacker)? {
            let permille =
                get_setting(&ctx.settings, b"battle_super_beast_hunter_df", 0x2bc)? as i64;

            damage = damage.wrapping_mul(permille);
            get_setting(&ctx.settings, b"battle_super_beast_hunter_df", 0x2bc)?;
            damage = ops::div_1000(damage);
        }

        if has_sage_slayer(ctx, 0, target)? && is_sage(ctx, 1, attacker)? {
            let permille =
                get_setting(&ctx.settings, b"battle_super_sage_hunter_damage2", 0x2bc)? as i64;

            damage = damage.wrapping_mul(permille);
            get_setting(&ctx.settings, b"battle_super_sage_hunter_damage2", 0x2bc)?;
            damage = ops::div_1000(damage);
        }

        let kaijin_combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x19, unit_id)?;

        if kaijin_combo > 0 && get_trait_kaijin(ctx, 1, attacker)? {
            damage = ops::div_wide(damage.wrapping_mul(1000), kaijin_combo as u32 as i64)
                .ok_or(Fault::divide(kaijin_combo as i64))?;
        }

        if get_barrier_hp(ctx, 0, target)? > 0 && barrier_broke != 0 {
            set_barrier_state(ctx, 0, target, 2)?;
        }

        if get_shield_hp(ctx, 0, target)? > 0 && shield_pierced != 0 {
            set_shield_state(ctx, 0, target, 2)?;
        }
    }

    match source {
        1 if get_wave_resist_pct(ctx, 0, target)? > 0 => {
            let resist = get_wave_resist_pct(ctx, 0, target)?;

            damage = damage.wrapping_mul(100i32.wrapping_sub(resist) as i64);
            get_wave_resist_pct(ctx, 0, target)?;
            damage = ops::div_100(damage);
        }
        3 if get_explosion_resist_pct(ctx, 0, target)? > 0 => {
            let resist = get_explosion_resist_pct(ctx, 0, target)?;

            damage = damage.wrapping_mul(100i32.wrapping_sub(resist) as i64);
            get_explosion_resist_pct(ctx, 0, target)?;
            damage = ops::div_100(damage);
        }
        2 if get_surge_resist_pct(ctx, 0, target)? > 0 => {
            let resist = get_surge_resist_pct(ctx, 0, target)?;

            damage = damage.wrapping_mul(100i32.wrapping_sub(resist) as i64);
            get_surge_resist_pct(ctx, 0, target)?;
            damage = ops::div_100(damage);
        }
        _ => {}
    }

    if form >= 2 {
        let defense = get_orb_value_vs_trait(ctx, unit_id, 1, 0, &traits, 0)?;

        if defense > 0 {
            damage = ops::div_100(damage.wrapping_mul(100i32.wrapping_sub(defense) as i64));
        }
    }

    let foundation_id = get_foundation_part_id(ctx)?;
    let mut foundation_level = get_foundation_part_level(ctx, foundation_id)?;
    let style_id = get_style_part_id(ctx)?;
    let mut style_level = get_style_part_level(ctx, style_id)?;
    let fixed_lineup = has_fixed_lineup(ctx, -1, -1, -1)?;

    foundation_level = foundation_level.wrapping_add(1);
    style_level = style_level.wrapping_add(1);

    if fixed_lineup & (ctx.i32_at(AppContext::LINEUP_CANNON_LEVEL)? != -1) {
        foundation_level = 1;
        style_level = 1;
    }

    if target != 0 {
        if is_red(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x64, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_floating(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x65, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_dark(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x66, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_metal(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x67, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_angel(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x68, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_alien(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x69, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_zombie(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6a, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_relic(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6b, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_traitless(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6c, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_witch(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6d, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_eva_angel(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6e, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }

        if is_aku(ctx, 1, attacker)? {
            let reduction =
                get_cannon_effect(get_foundation_part_rec(ctx)?, 0x6f, foundation_level)?;

            if reduction != 0 {
                damage = ops::div_10000(
                    damage.wrapping_mul(10000i32.wrapping_sub(reduction) as i64),
                );
            }
        }
    }

    let metal = is_metal(ctx, 0, target)?;
    let floored = if damage > 0 { damage } else { 0 };
    let mut dealt = 1i64;

    if damage < 2 {
        dealt = floored;
    }

    if crit != 0 {
        dealt = floored;
    }

    if !metal {
        dealt = floored;
    }

    let targets = does_target(ctx, 1, attacker, target)?;

    if target != 0 && targets as u8 & p_toxic != 0 {
        if get_toxic_immune(ctx, 0, target)? {
            start_immune_vfx(ctx, 0, target)?;
        } else {
            let max_hp = get_max_hp(ctx, 0, target)?;
            let mut toxic = max_i32(
                ops::div_100(get_toxic_damage(ctx, 1, attacker)?.wrapping_mul(max_hp)),
                1,
            );
            let resist = get_toxic_resist_pct(ctx, 0, target)?;

            toxic = max_i32(
                ops::div_100((100i32.wrapping_sub(resist) as i64).wrapping_mul(toxic as i64))
                    as i32,
                1,
            );

            let reduction = get_cannon_effect(get_resist_part_rec(ctx)?, 0xcd, style_level)?;
            let mut toxic = toxic as i64;

            if reduction != 0 {
                toxic = ops::div_10000(
                    (10000i32.wrapping_sub(reduction) as i64).wrapping_mul(toxic),
                );
            }

            set_toxic_vfx(ctx, 0, target, 1)?;
            get_toxic_resist_pct(ctx, 0, target)?;
            dealt = dealt.wrapping_add(toxic);
        }
    }

    let immune = match source {
        1 => get_wave_immune(ctx, 0, target)?,
        3 => get_explosion_immune(ctx, 0, target)?,
        2 => get_surge_immune(ctx, 0, target)?,
        _ => false,
    };

    if immune {
        ctx.set_i32_at(
            AppContext::entity_field(0, target, Entity::WAVE_IMMUNE_VFX_FRAME),
            0,
        )?;
        ctx.set_i32_at(
            AppContext::entity_field(0, target, Entity::WAVE_IMMUNE_VFX_ACTIVE),
            1,
        )?;
        dealt = 0;
    }

    attack_dmg_dispatch(ctx, 0, target, dealt as i32)?;

    Ok(true)
}
