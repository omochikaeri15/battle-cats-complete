use crate::{Fault, operation};

use super::{
    AppContext, add_drain_pct, add_score_hit_mask, battle_not_finishing, call_rng,
    count_target_traits, get_cannon_effect, get_cat_combo_bonus, get_curse_duration,
    get_curse_immune, get_curse_resist_pct, get_drain_immune, get_drain_percent,
    get_freeze_duration, get_freeze_immune, get_freeze_resist_pct, get_knockback_immune,
    get_resist_part_rec, get_score_bonus, get_setting, get_slot_unit_id, get_slow_duration,
    get_slow_immune, get_slow_resist_pct, get_style_part_id, get_style_part_level,
    get_treasure_capped, get_warp_anchor, get_warp_distance, get_warp_duration, get_warp_immune,
    get_warp_resist_pct, get_warp_span, get_weaken_duration, get_weaken_immune, get_weaken_pct,
    get_weaken_resist_pct, has_fixed_lineup, has_sage_slayer, is_sage, read_flag,
    scored_map_pays_money, set_curse_length, set_curse_timer, set_freeze_length, set_freeze_timer,
    set_kb_proc_hit, set_sage_kb_resist_pct, set_slow_length, set_slow_timer, set_warp_distance,
    set_warp_timer, set_weaken_active, set_weaken_active_pct, set_weaken_timer, slot_occupied,
    start_immune_vfx, turn_on_proc_badge,
};

pub fn attack_proc_dispatch(
    ctx: &mut AppContext,
    faction: i32,
    attacker: i32,
    target: i32,
    treasure: i32,
    p_knockback: i32,
    p_freeze: i32,
    p_slow: i32,
    p_weaken: i32,
    p_warp: u8,
    p_curse: u8,
    p_drain: u8,
) -> Result<(), Fault> {
    let other = 1i32.wrapping_sub(faction);
    let part_id = get_style_part_id(ctx)?;
    let part_level = get_style_part_level(ctx, part_id)?.wrapping_add(1);
    let unit_id = get_slot_unit_id(ctx, faction, attacker)?;
    let fixed_lineup = has_fixed_lineup(ctx, -1, -1, -1)?;
    let mut level = 1;

    if ctx.i32_at(AppContext::LINEUP_CANNON_LEVEL)? == -1 {
        level = part_level;
    }

    if !fixed_lineup {
        level = part_level;
    }

    if p_knockback == 1 {
        if get_knockback_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x8, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            set_kb_proc_hit(ctx, other, target, 1)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_knockback", 0x32)?;

                    set_sage_kb_resist_pct(ctx, other, target, resist)?;
                } else {
                    set_sage_kb_resist_pct(ctx, other, target, 0)?;
                }
            } else {
                if !has_sage_slayer(ctx, faction, attacker)? && is_sage(ctx, other, target)? {
                    let resist = get_setting(&ctx.settings, b"battle_super_sage_knockback", 0x32)?;

                    set_sage_kb_resist_pct(ctx, other, target, resist)?;
                } else {
                    set_sage_kb_resist_pct(ctx, other, target, 0)?;
                }

                if slot_occupied(ctx, faction, attacker)? == 2 && battle_not_finishing(ctx)? {
                    let traits = count_target_traits(ctx, faction, attacker)?;
                    let bonus = get_score_bonus(ctx, 0x3, traits)?;

                    add_score_hit_mask(ctx, other, target, bonus)?;
                }
            }
        }
    }

    let treasure = get_treasure_capped(ctx, &ctx.treasure_store, treasure, 0x64)?;

    if p_freeze == 1 {
        if get_freeze_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x6, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            let mut duration = get_freeze_duration(ctx, faction, attacker)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if get_freeze_resist_pct(ctx, other, target)? != 0 {
                    let resist = get_freeze_resist_pct(ctx, other, target)?;

                    get_freeze_resist_pct(ctx, other, target)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                let reduction = get_cannon_effect(get_resist_part_rec(ctx)?, 0xca, level)?;

                if reduction != 0 {
                    duration = operation::div_10000(
                        10000i32.wrapping_sub(reduction).wrapping_mul(duration),
                    );
                }

                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_freeze", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_hunter_freeze", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }
            } else {
                duration = operation::div_1500(duration.wrapping_mul(treasure.wrapping_add(0x5dc)));
                duration = operation::div_100(
                    get_cat_combo_bonus(ctx, &ctx.combo_store, 0x13, unit_id)?
                        .wrapping_add(100)
                        .wrapping_mul(duration),
                );
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0x13, unit_id)?;

                if !has_sage_slayer(ctx, faction, attacker)? && is_sage(ctx, other, target)? {
                    let resist = get_setting(&ctx.settings, b"battle_super_sage_freeze", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_freeze", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                if slot_occupied(ctx, faction, attacker)? == 2 && battle_not_finishing(ctx)? {
                    let traits = count_target_traits(ctx, faction, attacker)?;
                    let bonus = get_score_bonus(ctx, 0x1, traits)?;

                    add_score_hit_mask(ctx, other, target, bonus)?;
                }
            }

            set_freeze_timer(ctx, other, target, duration)?;
            set_freeze_length(ctx, other, target, duration)?;
            turn_on_proc_badge(ctx, other, target, 1)?;
        }
    }

    if p_slow == 1 {
        if get_slow_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x7, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            let mut duration = get_slow_duration(ctx, faction, attacker)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if get_slow_resist_pct(ctx, other, target)? != 0 {
                    let resist = get_slow_resist_pct(ctx, other, target)?;

                    get_slow_resist_pct(ctx, other, target)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                let reduction = get_cannon_effect(get_resist_part_rec(ctx)?, 0xc8, level)?;

                if reduction != 0 {
                    duration = operation::div_10000(
                        10000i32.wrapping_sub(reduction).wrapping_mul(duration),
                    );
                }

                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_slow", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_hunter_slow", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }
            } else {
                duration = operation::div_1500(duration.wrapping_mul(treasure.wrapping_add(0x5dc)));
                duration = operation::div_100(
                    get_cat_combo_bonus(ctx, &ctx.combo_store, 0x12, unit_id)?
                        .wrapping_add(100)
                        .wrapping_mul(duration),
                );
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0x12, unit_id)?;

                if !has_sage_slayer(ctx, faction, attacker)? && is_sage(ctx, other, target)? {
                    let resist = get_setting(&ctx.settings, b"battle_super_sage_slow", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_slow", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                if slot_occupied(ctx, faction, attacker)? == 2 && battle_not_finishing(ctx)? {
                    let traits = count_target_traits(ctx, faction, attacker)?;
                    let bonus = get_score_bonus(ctx, 0x2, traits)?;

                    add_score_hit_mask(ctx, other, target, bonus)?;
                }
            }

            set_slow_timer(ctx, other, target, duration)?;
            set_slow_length(ctx, other, target, duration)?;
            turn_on_proc_badge(ctx, other, target, 1)?;
        }
    }

    if p_weaken == 1 {
        if get_weaken_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x5, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            let mut duration = get_weaken_duration(ctx, faction, attacker)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if get_weaken_resist_pct(ctx, other, target)? != 0 {
                    let resist = get_weaken_resist_pct(ctx, other, target)?;

                    get_weaken_resist_pct(ctx, other, target)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                let reduction = get_cannon_effect(get_resist_part_rec(ctx)?, 0xcc, level)?;

                if reduction != 0 {
                    duration = operation::div_10000(
                        10000i32.wrapping_sub(reduction).wrapping_mul(duration),
                    );
                }

                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_weaken", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_hunter_weaken", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }
            } else {
                duration = operation::div_1500(duration.wrapping_mul(treasure.wrapping_add(0x5dc)));
                duration = operation::div_100(
                    get_cat_combo_bonus(ctx, &ctx.combo_store, 0x14, unit_id)?
                        .wrapping_add(100)
                        .wrapping_mul(duration),
                );
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0x14, unit_id)?;

                if !has_sage_slayer(ctx, faction, attacker)? && is_sage(ctx, other, target)? {
                    let resist = get_setting(&ctx.settings, b"battle_super_sage_weaken", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_weaken", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                if slot_occupied(ctx, faction, attacker)? == 2 && scored_map_pays_money(ctx)? {
                    let traits = count_target_traits(ctx, faction, attacker)?;
                    let bonus = get_score_bonus(ctx, 0x0, traits)?;

                    add_score_hit_mask(ctx, other, target, bonus)?;
                }
            }

            set_weaken_timer(ctx, other, target, duration)?;
            set_weaken_active(ctx, other, target, duration)?;

            let weaken_pct = get_weaken_pct(ctx, faction, attacker)?;

            set_weaken_active_pct(ctx, other, target, weaken_pct)?;
            turn_on_proc_badge(ctx, other, target, 2)?;
        }
    }

    if p_warp != 0 {
        if get_warp_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;
        } else {
            let mut duration = get_warp_duration(ctx, faction, attacker)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if get_warp_resist_pct(ctx, other, target)? != 0 {
                    let resist = get_warp_resist_pct(ctx, other, target)?;

                    get_warp_resist_pct(ctx, other, target)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_warp", 0x46)?;

                    get_setting(&ctx.settings, b"battle_super_sage_hunter_warp", 0x46)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }
            }

            set_warp_timer(ctx, other, target, duration)?;

            let anchor = get_warp_anchor(ctx, faction, attacker)?;
            let span = get_warp_span(ctx, faction, attacker)?;
            let bound = span
                .wrapping_sub(get_warp_anchor(ctx, faction, attacker)?)
                .wrapping_add(1);
            let distance = anchor.wrapping_add(call_rng(ctx, bound));

            set_warp_distance(ctx, other, target, distance)?;
            get_warp_distance(ctx, other, target)?;
        }
    }

    if p_curse != 0 {
        if get_curse_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x9, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            let mut duration = get_curse_duration(ctx, faction, attacker)?;

            if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
                if get_curse_resist_pct(ctx, other, target)? != 0 {
                    let resist = get_curse_resist_pct(ctx, other, target)?;

                    get_curse_resist_pct(ctx, other, target)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                let reduction = get_cannon_effect(get_resist_part_rec(ctx)?, 0xce, level)?;

                if reduction != 0 {
                    duration = operation::div_10000(
                        10000i32.wrapping_sub(reduction).wrapping_mul(duration),
                    );
                }

                if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                    let resist =
                        get_setting(&ctx.settings, b"battle_super_sage_hunter_curse", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_hunter_curse", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }
            } else {
                duration = operation::div_1500(duration.wrapping_mul(treasure.wrapping_add(0x5dc)));

                if !has_sage_slayer(ctx, faction, attacker)? && is_sage(ctx, other, target)? {
                    let resist = get_setting(&ctx.settings, b"battle_super_sage_curse", 0x32)?;

                    get_setting(&ctx.settings, b"battle_super_sage_curse", 0x32)?;
                    duration =
                        operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(duration));
                }

                if slot_occupied(ctx, faction, attacker)? == 2 && scored_map_pays_money(ctx)? {
                    let traits = count_target_traits(ctx, faction, attacker)?;
                    let bonus = get_score_bonus(ctx, 0x4, traits)?;

                    add_score_hit_mask(ctx, other, target, bonus)?;
                }
            }

            set_curse_timer(ctx, other, target, duration)?;
            set_curse_length(ctx, other, target, duration)?;
            turn_on_proc_badge(ctx, other, target, 4)?;
        }
    }

    if target != 0 && p_drain != 0 {
        if get_drain_immune(ctx, other, target)? {
            start_immune_vfx(ctx, other, target)?;

            let scoring = battle_not_finishing(ctx)?;

            if faction != 0 && scoring {
                let bonus = get_score_bonus(ctx, 0x13, 1)?;

                add_score_hit_mask(ctx, other, target, bonus)?;
            }
        } else {
            let mut percent = get_drain_percent(ctx, faction, attacker)?;

            if has_sage_slayer(ctx, other, target)? && is_sage(ctx, faction, attacker)? {
                let resist =
                    get_setting(&ctx.settings, b"battle_super_sage_hunter_knockback", 0x32)?;

                percent = operation::div_100(100i32.wrapping_sub(resist).wrapping_mul(percent));
            }

            add_drain_pct(ctx, other, target, percent)?;
        }
    }

    Ok(())
}
