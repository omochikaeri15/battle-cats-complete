use crate::{operation, Fault};

use super::{
    add_medal_progress, add_money, award_event_points, conjurer_on_field, deck_slot_filled, get_altar_level_cap, get_button_unit_form, get_button_unit_id, get_button_unit_row,
    get_castle_enemy_row, get_deck_cooldown, get_effective_deploy_cost, get_global_map_id, get_money, get_orb_value_max, get_pos_x, get_setting, get_slot_unit_id,
    get_special_rule, get_special_rule_params, get_standing_range, get_unit_max_level, get_unit_rarity, get_unit_recharge, has_fixed_lineup, is_deploy_blocked, is_score_stage,
    level_cell_base, level_cell_plus, max_i32, min_i32, orb_deploy_condition, play_sound, set_attack_end_mode, set_attacks_remaining, set_boss_wave_immune,
    set_conjure_deck_slot, set_deck_cooldown, set_dodge_chance, set_dodge_duration, set_entity_state, set_paid_cost, set_pos_x, slot_conjure_ready, slot_deploy_permitted,
    slot_occupied, sound_manager, spawn_entity, stage_not_sealed, stat_conjure_unit_id, AppContext, CatStats, CAT_STATS, CAT_STATS_FORM_STRIDE, CAT_STATS_UNIT_STRIDE,
};

pub fn deploy_unit(ctx: &mut AppContext, faction: i32, slot: i32, fail_sound: u8) -> Result<(), Fault> {
    const SITE: &str = "deploy_unit";

    'refused: {
        if !slot_deploy_permitted(ctx, faction, slot, 1)? {
            ctx.set_i32_at(AppContext::DEPLOY_NOTICE_TIMER, 0x1e)?;
            ctx.set_i32_at(AppContext::DEPLOY_NOTICE_KIND, 2)?;

            break 'refused;
        }

        if !deck_slot_filled(ctx, faction, slot)? {
            break 'refused;
        }

        let cost = get_effective_deploy_cost(ctx, faction, slot)?;
        let conjure = slot_conjure_ready(ctx, faction, slot)?;
        let wallet = AppContext::faction_flags(faction);
        let cell = ((slot as i64) * 4) as usize;

        if conjure && ctx.i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_CONJURE_LOCKOUT))? > 0 {
            break 'refused;
        }

        if !((get_deck_cooldown(ctx, wallet, slot)? == 0) | conjure) {
            break 'refused;
        }

        if !((get_money(ctx, wallet)? >= cost) | conjure) {
            break 'refused;
        }

        if is_deploy_blocked(ctx, slot)? {
            if !conjure {
                break 'refused;
            }

            if !conjurer_on_field(ctx, faction, slot, 1)? {
                break 'refused;
            }
        }

        let mut base_cap = 0x3e7i32;
        let mut plus_cap = 0x3e7i32;

        if !stage_not_sealed(ctx, get_castle_enemy_row(ctx)?.wrapping_add(-2))? {
            let cap = get_altar_level_cap(ctx, get_castle_enemy_row(ctx)?.wrapping_add(-2))?;

            plus_cap = if cap != -1 { cap } else { 0x3e7 };
            base_cap = 0;
        }

        'spent: {
            if conjure {
                if !conjurer_on_field(ctx, faction, slot, 1)? {
                    if get_deck_cooldown(ctx, wallet, slot)? == 0 && ctx.u8_at(wallet.wrapping_add(slot as i64 as usize).wrapping_add(AppContext::WALLET_SPIRIT_USED))? != 0 {
                        ctx.set_i32_at(AppContext::DEPLOY_NOTICE_TIMER, 0x1e)?;
                        ctx.set_i32_at(AppContext::DEPLOY_NOTICE_KIND, 3)?;
                    }

                    break 'refused;
                }

                let spirit_button = slot.wrapping_add(0xb);
                let plus = min_i32(level_cell_plus(ctx, ((get_button_unit_id(ctx, faction, slot)? as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize)?, plus_cap);
                let base = min_i32(level_cell_base(ctx, ((get_button_unit_id(ctx, faction, slot)? as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize)?, base_cap);
                let mut level = base.wrapping_add(plus);
                let mut last = level;

                if level >= get_unit_max_level(ctx, get_button_unit_id(ctx, faction, spirit_button)?)? {
                    level = get_unit_max_level(ctx, get_button_unit_id(ctx, faction, spirit_button)?)?.wrapping_sub(1);
                }

                let mut announced = false;
                let mut scan = 1i32;

                while scan != 51 {
                    if slot_occupied(ctx, 0, scan)? == 0 || get_slot_unit_id(ctx, faction, scan)? != get_button_unit_id(ctx, faction, slot)? {
                        scan += 1;

                        continue;
                    }

                    let row = get_button_unit_row(ctx, faction, spirit_button)?;
                    let z_min_row = get_button_unit_row(ctx, faction, spirit_button)? as i64;
                    let z_min_form = get_button_unit_form(ctx, faction, spirit_button)? as i64;
                    let z_min = ctx.i32_at((z_min_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_min_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MINIMUM_Z_LAYER as i64) as usize)?;
                    let z_max_row = get_button_unit_row(ctx, faction, spirit_button)? as i64;
                    let z_max_form = get_button_unit_form(ctx, faction, spirit_button)? as i64;
                    let z_max = ctx.i32_at((z_max_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_max_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MAXIMUM_Z_LAYER as i64) as usize)?;
                    let form = get_button_unit_form(ctx, faction, spirit_button)?;

                    last = spawn_entity(ctx, faction, row, level, z_min, z_max, form, 0)?;

                    if last == -1 {
                        if fail_sound != 0 {
                            play_sound(sound_manager(ctx)?, 0xf, None);
                        }

                        return Ok(());
                    }

                    ctx.set_block_at::<1>(wallet.wrapping_add(slot as i64 as usize).wrapping_add(AppContext::WALLET_SPIRIT_USED), [1])?;
                    set_conjure_deck_slot(ctx, faction, last, slot)?;

                    if !announced {
                        play_sound(sound_manager(ctx)?, 0xa2, None);
                        announced = true;
                    }

                    let mut x = get_pos_x(ctx, faction, scan)?.wrapping_sub(get_setting(&ctx.settings, b"summon_position", 0)?.wrapping_shl(2));

                    if x.wrapping_sub(get_standing_range(ctx, faction, last)?) <= 0xc7f {
                        x = get_standing_range(ctx, faction, last)?.wrapping_add(0xc80);
                    }

                    let limit = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0xc80);

                    set_pos_x(ctx, faction, last, if x < limit { x } else { limit })?;
                    set_attacks_remaining(ctx, faction, last, 1)?;
                    set_attack_end_mode(ctx, faction, last, 2)?;
                    set_dodge_chance(ctx, faction, last, 0x64)?;
                    set_dodge_duration(ctx, faction, last, 0x2710)?;
                    set_boss_wave_immune(ctx, faction, last, 1)?;
                    set_entity_state(ctx, faction, last, 2)?;
                    scan += 1;
                }

                if last < 0 {
                    if fail_sound != 0 {
                        play_sound(sound_manager(ctx)?, 0xf, None);
                    }

                    return Ok(());
                }

                break 'spent;
            }

            let plus = min_i32(level_cell_plus(ctx, ((get_button_unit_id(ctx, faction, slot)? as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize)?, plus_cap);
            let base = min_i32(level_cell_base(ctx, ((get_button_unit_id(ctx, faction, slot)? as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize)?, base_cap);
            let mut level = base.wrapping_add(plus);

            if has_fixed_lineup(ctx, -1, -1, -1)? && !ctx.fixed_lineup_store.units.is_empty() {
                let mut listed = 0usize;

                while listed < ctx.fixed_lineup_store.units.len() {
                    let unit = ctx.fixed_lineup_store.units.get(listed).ok_or(Fault::IndexOutOfRange { site: SITE, index: listed as i64, limit: 0 })?;
                    let (unit_id, fixed_level) = (unit.unit_id, unit.plus_level.wrapping_add(unit.level));

                    if unit_id == get_button_unit_id(ctx, faction, slot)? {
                        level = fixed_level;

                        break;
                    }

                    listed += 1;
                }
            }

            let row = get_button_unit_row(ctx, faction, slot)?;
            let z_min_row = get_button_unit_row(ctx, faction, slot)? as i64;
            let z_min_form = get_button_unit_form(ctx, faction, slot)? as i64;
            let z_min = ctx.i32_at((z_min_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_min_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MINIMUM_Z_LAYER as i64) as usize)?;
            let z_max_row = get_button_unit_row(ctx, faction, slot)? as i64;
            let z_max_form = get_button_unit_form(ctx, faction, slot)? as i64;
            let z_max = ctx.i32_at((z_max_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_max_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MAXIMUM_Z_LAYER as i64) as usize)?;
            let form = get_button_unit_form(ctx, faction, slot)?;
            let spawned = spawn_entity(ctx, faction, row, level, z_min, z_max, form, 0)?;

            if faction == 0 && spawned >= 0 {
                let map_id = get_global_map_id(ctx, 0)?;

                if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 8)? {
                    let params = params.to_vec();
                    let rarity = get_unit_rarity(ctx, get_button_unit_id(ctx, 0, slot)?)?;
                    let mask = *params.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

                    if (mask as u32 >> (rarity as u32 & 0x1f)) & 1 != 0 {
                        let mut queued = 0i32;

                        loop {
                            if queued >= *params.get(1).ok_or(Fault::IndexOutOfRange { site: SITE, index: 1, limit: params.len() as i64 })? {
                                break;
                            }

                            let gap = *params.get(2).ok_or(Fault::IndexOutOfRange { site: SITE, index: 2, limit: params.len() as i64 })?;

                            queued = queued.wrapping_add(1);
                            ctx.deploy_queue.push(((gap.wrapping_mul(queued) as u32 as u64) << 0x20).wrapping_add(slot as u32 as u64));
                        }
                    }
                }

                set_paid_cost(ctx, 0, spawned, cost)?;

                if is_score_stage(ctx.event_items.as_ref()) {
                    let unit_id = get_button_unit_id(ctx, 0, slot)?;
                    let form = get_button_unit_form(ctx, 0, slot)? as i64;
                    let listed_cost = ctx.i32_at(((unit_id.wrapping_add(2) as i64) * CAT_STATS_UNIT_STRIDE as i64 + form * CAT_STATS_FORM_STRIDE as i64 + (CAT_STATS + CatStats::EOC1_COST) as i64) as usize)?;
                    let boosted = ctx.i32_at(AppContext::EVENT_POINT_BOOST)?.wrapping_add(2).wrapping_mul(operation::div_100(listed_cost as i64) as i32);
                    let args = [operation::div_2(boosted)];
                    let store = ctx.event_items.as_mut().ok_or(Fault::NullPointer { site: SITE })?;

                    award_event_points(store, 0, &args)?;
                }
            }

            if spawned < 0 {
                if fail_sound != 0 {
                    play_sound(sound_manager(ctx)?, 0xf, None);
                }

                return Ok(());
            }

            let unit_id = get_button_unit_id(ctx, faction, slot)?;
            let form = get_button_unit_form(ctx, faction, slot)?;

            if stat_conjure_unit_id(ctx, faction, unit_id, form)? >= 0 {
                ctx.set_i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_CONJURE_READY), 1)?;
                ctx.set_i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_CONJURE_LOCKOUT), 0xf)?;
                ctx.set_block_at::<1>(wallet.wrapping_add(slot as i64 as usize).wrapping_add(AppContext::WALLET_SPIRIT_USED), [0])?;
                ctx.set_i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_CONJURE_TIMER), 0)?;
            }

            let paid = get_effective_deploy_cost(ctx, faction, slot)?;

            ctx.set_i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_ESCALATING_COSTS), paid)?;

            let deployed = ctx.i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_DEPLOY_COUNTS))?;

            ctx.set_i32_at(wallet.wrapping_add(cell).wrapping_add(AppContext::WALLET_DEPLOY_COUNTS), deployed.wrapping_add(1))?;

            let total = ctx.i32_at(AppContext::DEPLOY_LIMIT_TOTAL)?;

            ctx.set_i32_at(AppContext::DEPLOY_LIMIT_TOTAL, total.wrapping_add(1))?;
        }

        ctx.set_i32_at(AppContext::CPU_PENDING_ACTION, 0)?;

        if !conjure {
            add_money(ctx, wallet, cost.wrapping_neg())?;

            let mut recharge = 0i32;

            if ctx.i32_at(AppContext::BABY_BOOM_ACTIVE)? != 1 {
                recharge = get_unit_recharge(ctx, faction, slot)?;

                let floor = min_i32(recharge, 0x3c);

                if faction == 0 && get_button_unit_form(ctx, 0, slot)? >= 2 && orb_deploy_condition(ctx, wallet, 0, slot)? {
                    let map_id = get_global_map_id(ctx, 0)?;

                    if !get_special_rule(ctx, &ctx.special_rules, map_id, 1)? {
                        let unit_id = get_button_unit_id(ctx, 0, slot)?;
                        let saved = get_orb_value_max(ctx, unit_id, 0x13, 0, 0)?;

                        recharge = max_i32(operation::div_100(0x64i32.wrapping_sub(saved).wrapping_mul(recharge)), floor);
                    }
                }
            }

            set_deck_cooldown(ctx, wallet, slot, recharge, 1)?;
        }

        play_sound(sound_manager(ctx)?, 0x13, None);

        return add_medal_progress(ctx, 0, operation::div_100(cost as i64) as i32);
    }

    play_sound(sound_manager(ctx)?, 0xf, None);

    Ok(())
}
