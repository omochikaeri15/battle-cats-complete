use crate::{Fault, operation};

use super::{
    AppContext, Base, Debris, ENEMY_STATS, ENEMY_STATS_STRIDE, EnemyStats, Entity, KNOCKBACK_Y_ARC,
    RECOIL_Y_ARC, SurgeEvent, WaveRecord, abs_i32, add_entity_frame, add_hit_flash_timer,
    add_kill_count, add_money, add_pos_x, add_revive_count, add_revive_timer,
    advance_animation_frame, attack_dmg_dispatch, award_event_points, call_rng, can_push_back,
    cat_update, check_collision, get_anim_len, get_attacks_remaining, get_base_hp,
    get_battle_status, get_burrow_distance, get_burrow_start_x, get_cannon_base_damage,
    get_castle_id, get_castle_row, get_cat_combo_bonus, get_curse_timer, get_death_surge_anchor,
    get_death_surge_level, get_death_surge_mini, get_death_surge_span, get_enemy_money_drop,
    get_entity_button, get_entity_frame, get_entity_state, get_first_bounty_pct, get_freeze_timer,
    get_global_map_id, get_gudetama_soul, get_hit_flash_timer, get_hp, get_max_hp, get_no_revive,
    get_pos_x, get_revive_count, get_revive_hp, get_revive_time, get_revive_timer,
    get_sage_kb_resist_pct, get_score_time_limit, get_score_value, get_setting, get_shield_hp,
    get_shield_max, get_shield_regen, get_slow_timer, get_soul_anim_type, get_spawn_serial,
    get_special_rule, get_speed, get_treasure_value, get_unit_anim, get_weaken_timer, is_alien,
    is_angel, is_dark, is_floating, is_metal, is_red, is_score_stage, is_scored_stage,
    is_touchable_thunk, is_zombie, keep_in_bound, latch_battle_event, max_i32, min_i32,
    no_more_attacks, pending_strike_hit, play_sound, play_sound_in_battle, read_flag, roll_procs,
    scored_map_pays_money, set_crit_vfx, set_drain_pct, set_entity_frame, set_entity_state,
    set_first_bounty_pct, set_frame_damage, set_hp, set_metal_killer_vfx, set_prev_curse_timer,
    set_prev_freeze_timer, set_prev_slow_timer, set_prev_weaken_timer, set_proc_badge,
    set_revive_timer, set_savage_blow_vfx, set_score_hit_mask, set_shield_hp, set_shield_state,
    set_shield_vfx, set_shield_vfx_frame, set_toxic_vfx, slot_occupied, sound_manager,
    spawn_warp_tick, std_map_int_maanim_subscript_2, std_map_int_vector_erase,
    std_map_int_vector_subscript, std_vector_int_push_back, std_vector_int_push_back_2,
};

#[derive(Default)]
pub struct CounterSurgeEvent {
    pub faction: i32,
    pub slot: i32,
    pub state: i32,
    pub frame: i32,
    pub x: i32,
    pub level: i32,
    pub anchor: i32,
    pub span: i32,
    pub mini: bool,
}

pub fn enemy_update(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    let mut first_bounty_slots: Vec<i32> = Vec::new();

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        return cat_update(ctx, faction);
    }

    let other = 1i32.wrapping_sub(faction);
    let base = AppContext::entity_field(faction, 0, 0);
    let credit = faction as i64 as usize;

    let mut slot = 0i64;

    loop {
        'slot: {
            if slot_occupied(ctx, faction, slot as i32)? == 0 {
                break 'slot;
            }

            set_frame_damage(ctx, faction, slot as i32, 0)?;
            set_drain_pct(ctx, faction, slot as i32, 0)?;

            let freeze_timer = get_freeze_timer(ctx, faction, slot as i32)?;
            set_prev_freeze_timer(ctx, faction, slot as i32, freeze_timer)?;

            let slow_timer = get_slow_timer(ctx, faction, slot as i32)?;
            set_prev_slow_timer(ctx, faction, slot as i32, slow_timer)?;

            let weaken_timer = get_weaken_timer(ctx, faction, slot as i32)?;
            set_prev_weaken_timer(ctx, faction, slot as i32, weaken_timer)?;

            let curse_timer = get_curse_timer(ctx, faction, slot as i32)?;
            set_prev_curse_timer(ctx, faction, slot as i32, curse_timer)?;

            set_score_hit_mask(ctx, faction, slot as i32, 0)?;
            set_crit_vfx(ctx, faction, slot as i32, 0)?;
            set_savage_blow_vfx(ctx, faction, slot as i32, 0)?;
            set_toxic_vfx(ctx, faction, slot as i32, 0)?;
            set_metal_killer_vfx(ctx, faction, slot as i32, 0)?;

            if get_hit_flash_timer(ctx, faction, slot as i32)? > 0 {
                add_hit_flash_timer(ctx, faction, slot as i32, -1)?;
            }

            if slot == 0 {
                if get_base_hp(ctx, faction)? <= 0 {
                    ctx.set_i32_at(base.wrapping_add(Base::STATE), 2)?;

                    let counter = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;
                    let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
                    let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
                    let size_again = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
                    let ring =
                        0x32i32.wrapping_sub(counter.wrapping_sub(
                            (operation::div_5(counter as i64) as i32).wrapping_mul(5),
                        ));
                    let width = operation::div_100(size.wrapping_mul(0x49c) as i64) as i32;
                    let depth = operation::div_100(size_again.wrapping_mul(offset_x) as i64) as i32;
                    let span = depth.wrapping_add(width);
                    let half = (((span as u32) >> 0x1f) as i32).wrapping_add(span) >> 1;

                    ctx.set_i32_at(AppContext::DRAW_TEMP_1, x.wrapping_sub(half))?;

                    let record = AppContext::ENEMY_DEBRIS.wrapping_add(
                        (ring as u32 as usize).wrapping_mul(AppContext::DEBRIS_STRIDE),
                    );

                    ctx.set_i32_at(record.wrapping_add(Debris::TIMER), 0xc)?;

                    let blast_x = ctx.i32_at(AppContext::DRAW_TEMP_1)?;
                    let scatter = call_rng(ctx, 0xf1);

                    ctx.set_i32_at(
                        record.wrapping_add(Debris::POS_X),
                        blast_x
                            .wrapping_add(scatter.wrapping_mul(5).wrapping_mul(2))
                            .wrapping_add(-0x75f),
                    )?;

                    let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                    let scatter = call_rng(ctx, 0x143);

                    ctx.set_i32_at(
                        record.wrapping_add(Debris::POS_Y),
                        y.wrapping_sub(scatter.wrapping_mul(2).wrapping_mul(5))
                            .wrapping_add(-0x24b),
                    )?;
                    ctx.set_i32_at(record.wrapping_add(Debris::VARIANT), 0)?;
                } else if ctx.i32_at(base.wrapping_add(Base::STATE))? == 1 {
                    let frame = ctx.i32_at(base.wrapping_add(Entity::FRAME))?;

                    ctx.set_i32_at(base.wrapping_add(Entity::FRAME), frame.wrapping_sub(1))?;

                    if frame <= 1 {
                        ctx.set_block_at(base.wrapping_add(Base::STATE), [0u8; 8])?;
                    }
                }

                break 'slot;
            }

            let entity = AppContext::entity_field(faction, slot as i32, 0);
            let cooldown = ctx.i32_at(entity.wrapping_add(Entity::ATTACK_COOLDOWN))?;

            if cooldown > 0 {
                ctx.set_i32_at(
                    entity.wrapping_add(Entity::ATTACK_COOLDOWN),
                    cooldown.wrapping_sub(1),
                )?;
            }

            ctx.set_i32_at(entity.wrapping_add(Entity::CANNON_BLAST_HIT), 0)?;
            pending_strike_hit(ctx, faction, slot as i32, other)?;

            if ctx.i32_at(AppContext::CANNON_BLAST_ACTIVE)? == 1
                && is_touchable_thunk(ctx, faction, slot as i32, -1)?
            {
                let damage = get_cannon_base_damage(ctx, other)?.wrapping_mul(3);

                attack_dmg_dispatch(ctx, faction, slot as i32, damage)?;
                ctx.set_i32_at(entity.wrapping_add(Entity::CANNON_BLAST_HIT), 1)?;
            }

            if get_entity_state(ctx, faction, slot as i32)? == 0 {
                let mut target = 0i32;

                loop {
                    if check_collision(ctx, faction, slot as i32, target, 0)? {
                        break 'slot;
                    }

                    target += 1;

                    if target == 0x33 {
                        break;
                    }
                }

                if get_freeze_timer(ctx, faction, slot as i32)? > 0 {
                    break 'slot;
                }

                let mut speed = get_speed(ctx, faction, slot as i32)?;

                if get_slow_timer(ctx, faction, slot as i32)? > 0 {
                    speed = min_i32(speed, 1);
                }

                add_pos_x(ctx, faction, slot as i32, speed)?;
                advance_animation_frame(ctx, faction, slot as i32)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0xb {
                if get_freeze_timer(ctx, faction, slot as i32)? <= 0 {
                    add_entity_frame(ctx, faction, slot as i32, 1)?;
                }

                if get_entity_frame(ctx, faction, slot as i32)? >= 0x1e {
                    set_entity_state(ctx, faction, slot as i32, 0xc)?;
                    set_entity_frame(ctx, faction, slot as i32, 0)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0xc {
                add_entity_frame(ctx, faction, slot as i32, 1)?;

                let x = get_pos_x(ctx, faction, slot as i32)?;
                let start = get_burrow_start_x(ctx, faction, slot as i32)?;
                let surface = get_burrow_distance(ctx, faction, slot as i32)?.wrapping_add(start);

                if x <= surface {
                    let mut speed = get_speed(ctx, faction, slot as i32)?;

                    if get_slow_timer(ctx, faction, slot as i32)? > 0 {
                        speed = min_i32(speed, 1);
                    }

                    add_pos_x(ctx, faction, slot as i32, speed)?;
                } else {
                    set_entity_state(ctx, faction, slot as i32, 0xd)?;
                    set_entity_frame(ctx, faction, slot as i32, 0)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0xd {
                if get_freeze_timer(ctx, faction, slot as i32)? <= 0 {
                    add_entity_frame(ctx, faction, slot as i32, 1)?;
                }

                if get_entity_frame(ctx, faction, slot as i32)? >= 0x1e {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0xe {
                add_entity_frame(ctx, faction, slot as i32, 1)?;

                if get_revive_timer(ctx, faction, slot as i32)? > 0 {
                    add_revive_timer(ctx, faction, slot as i32, -1)?;

                    if get_revive_timer(ctx, faction, slot as i32)? == 0 {
                        set_entity_state(ctx, faction, slot as i32, 0xf)?;
                        set_entity_frame(ctx, faction, slot as i32, 0)?;
                    }
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0xf {
                add_entity_frame(ctx, faction, slot as i32, 1)?;

                if get_entity_frame(ctx, faction, slot as i32)? > 0x19 {
                    set_entity_state(ctx, faction, slot as i32, 0x10)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0x10 {
                add_entity_frame(ctx, faction, slot as i32, 1)?;

                if get_entity_frame(ctx, faction, slot as i32)? > 0x27 {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 3 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? != 0 {
                    if can_push_back(ctx, faction, slot as i32)? {
                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                        ctx.set_i32_at(entity.wrapping_add(Entity::POS_X), x.wrapping_add(-0x3c))?;
                        keep_in_bound(ctx, faction, slot as i32)?;
                    }

                    let frame = ctx.i32_at(entity.wrapping_add(Entity::FRAME))? as i64;
                    let lift =
                        *KNOCKBACK_Y_ARC
                            .get(frame as usize)
                            .ok_or(Fault::index_out_of_range(frame, 24))?;
                    let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;

                    ctx.set_i32_at(entity.wrapping_add(Entity::POS_Y), y.wrapping_add(lift))?;

                    break 'slot;
                }

                if get_attacks_remaining(ctx, faction, slot as i32)? == 0 {
                    no_more_attacks(ctx, faction, slot as i32)?;
                } else {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                }

                if get_hp(ctx, faction, slot as i32)? != 0 {
                    ctx.set_i32_at(entity.wrapping_add(Entity::DOUBLE_BOUNTY_STATE), 0)?;

                    if get_shield_max(ctx, faction, slot as i32)? <= 0 {
                        break 'slot;
                    }

                    if get_shield_hp(ctx, faction, slot as i32)? != 0 {
                        break 'slot;
                    }

                    let shield_max = get_shield_max(ctx, faction, slot as i32)?;
                    let restored = operation::div_100(
                        get_shield_regen(ctx, faction, slot as i32)?.wrapping_mul(shield_max)
                            as i64,
                    ) as i32;

                    set_shield_hp(ctx, faction, slot as i32, restored)?;
                    set_shield_state(ctx, faction, slot as i32, 0)?;
                    set_shield_vfx(ctx, faction, slot as i32, 3)?;
                    set_shield_vfx_frame(ctx, faction, slot as i32, 0)?;
                    play_sound_in_battle(ctx, 0x8a)?;

                    break 'slot;
                }

                if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 || get_battle_status(ctx)? != 4 {
                    play_sound(sound_manager(ctx)?, 0x17, None);
                }

                if is_zombie(ctx, faction, slot as i32)?
                    && get_revive_count(ctx, faction, slot as i32)? != 0
                    && !get_no_revive(ctx, faction, slot as i32)?
                {
                    if get_revive_count(ctx, faction, slot as i32)? > 0 {
                        add_revive_count(ctx, faction, slot as i32, -1)?;
                    }

                    set_entity_state(ctx, faction, slot as i32, 0xe)?;
                    set_entity_frame(ctx, faction, slot as i32, 0)?;

                    let revive_time = get_revive_time(ctx, faction, slot as i32)?;

                    set_revive_timer(ctx, faction, slot as i32, revive_time)?;

                    let max_hp = get_max_hp(ctx, faction, slot as i32)?;
                    let revived = operation::div_100(
                        get_revive_hp(ctx, faction, slot as i32)?.wrapping_mul(max_hp) as i64,
                    ) as i32;

                    set_hp(ctx, faction, slot as i32, revived)?;

                    break 'slot;
                }

                set_entity_state(ctx, faction, slot as i32, 4)?;
                set_entity_frame(ctx, faction, slot as i32, 0)?;

                let mut first_bounty_slot = -1i32;
                let mut bounty = 0i32;
                let serial = get_spawn_serial(ctx, faction, slot as i32)?;

                if ctx.attackers_by_serial[credit].contains_key(&serial) {
                    let serial = get_spawn_serial(ctx, faction, slot as i32)?;
                    let attackers =
                        std_map_int_vector_subscript(&mut ctx.attackers_by_serial[credit], &serial)
                            .to_vec();

                    bounty = 0;

                    for attacker_serial in attackers {
                        let mut attacker = 1i32;

                        loop {
                            if get_spawn_serial(ctx, other, attacker)? == attacker_serial {
                                add_kill_count(ctx, other, attacker, 1)?;

                                let percent = get_first_bounty_pct(ctx, other, attacker)?;

                                if percent > 0 && percent > bounty {
                                    first_bounty_slot = attacker;
                                    bounty = percent;
                                }
                            }

                            attacker += 1;

                            if attacker == 0x33 {
                                break;
                            }
                        }
                    }

                    let serial = get_spawn_serial(ctx, faction, slot as i32)?;

                    std_map_int_vector_erase(&mut ctx.attackers_by_serial[credit], &serial);
                }

                if is_scored_stage(ctx)? && get_battle_status(ctx)? != 4 {
                    ctx.set_block_at(AppContext::SCORE_CHANGED, 1u64.to_le_bytes())?;

                    let limit = get_score_time_limit(ctx)?;
                    let weighted = limit
                        .wrapping_add(limit)
                        .wrapping_sub(ctx.i32_at(AppContext::SCORE_ELAPSED)?)
                        .wrapping_mul(ctx.i32_at(entity.wrapping_add(Entity::SCORE_VALUE))?);
                    let time_bonus = operation::idiv(weighted, limit)
                        .ok_or(Fault::divide(limit as i64))?;
                    let occupant = ctx.i32_at(entity.wrapping_add(Entity::OCCUPANT))? as i64;
                    let drop = ctx.i32_at(
                        (occupant * ENEMY_STATS_STRIDE as i64
                            + (ENEMY_STATS + EnemyStats::CASH_DROP) as i64)
                            as usize,
                    )?;
                    let score = (operation::div_100(drop as i64) as i32)
                        .wrapping_add(time_bonus)
                        .wrapping_add(ctx.i32_at(AppContext::SCORE_TOTAL)?);

                    ctx.set_i32_at(
                        AppContext::SCORE_TOTAL,
                        if score >= 0x3b9ac9ff {
                            0x3b9ac9ff
                        } else {
                            score
                        },
                    )?;
                }

                if is_score_stage(ctx.event_items.as_ref()) && get_battle_status(ctx)? != 4 {
                    let occupant = ctx.i32_at(entity.wrapping_add(Entity::OCCUPANT))? as i64;
                    let drop = ctx.i32_at(
                        (occupant * ENEMY_STATS_STRIDE as i64
                            + (ENEMY_STATS + EnemyStats::CASH_DROP) as i64)
                            as usize,
                    )?;
                    let score_value = get_score_value(ctx, faction, slot as i32)?;
                    let args = [operation::div_100(drop as i64) as i32, score_value];
                    let store = ctx
                        .event_items
                        .as_mut()
                        .ok_or(Fault::null_pointer())?;

                    award_event_points(store, 1, &args)?;
                }

                'money: {
                    if is_scored_stage(ctx)? && !scored_map_pays_money(ctx)? {
                        break 'money;
                    }

                    let map_id = get_global_map_id(ctx, 0)?;

                    if get_special_rule(ctx, &ctx.special_rules, map_id, 0)? {
                        break 'money;
                    }

                    let drop = get_enemy_money_drop(ctx, faction, slot as i32, bounty)?;

                    add_money(ctx, AppContext::faction_flags(0), drop)?;

                    if bounty > 0 {
                        std_vector_int_push_back_2(&mut first_bounty_slots, &first_bounty_slot);
                    }
                }

                ctx.set_block_at(entity.wrapping_add(Entity::FREEZE_TIMER), [0u8; 8])?;
                ctx.set_block_at(entity.wrapping_add(Entity::FREEZE_LENGTH), [0u8; 8])?;
                ctx.set_block_at(entity.wrapping_add(Entity::WEAKEN_TIMER), [0u8; 8])?;
                ctx.set_i32_at(entity.wrapping_add(Entity::SURVIVE_USED), 0)?;
                set_proc_badge(ctx, faction, slot as i32, 0, 0)?;
                set_proc_badge(ctx, faction, slot as i32, 1, 0)?;
                set_proc_badge(ctx, faction, slot as i32, 2, 0)?;
                set_proc_badge(ctx, faction, slot as i32, 3, 0)?;
                set_proc_badge(ctx, faction, slot as i32, 4, 0)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 4
                || get_entity_state(ctx, faction, slot as i32)? == 0x15
            {
                let mut finished;
                let mut pair = 0usize;

                'pending: {
                    loop {
                        let first = AppContext::WAVE_RECORDS
                            .wrapping_add(pair.wrapping_mul(AppContext::WAVE_RECORD_STRIDE * 2));
                        let second = first.wrapping_add(AppContext::WAVE_RECORD_STRIDE);

                        if slot
                            == ctx.i32_at(first.wrapping_add(WaveRecord::OWNER_SLOT))? as u32 as i64
                            && ctx.i32_at(first.wrapping_add(WaveRecord::IN_USE))? == 2
                        {
                            finished = false;

                            break 'pending;
                        }

                        if slot
                            == ctx.i32_at(second.wrapping_add(WaveRecord::OWNER_SLOT))? as u32
                                as i64
                            && ctx.i32_at(second.wrapping_add(WaveRecord::IN_USE))? == 2
                        {
                            finished = false;

                            break 'pending;
                        }

                        pair += 1;

                        if pair == 100 {
                            break;
                        }
                    }

                    finished = false;

                    let mut index = 0usize;

                    while index != ctx.surge_events.len() {
                        let event = &ctx.surge_events[index];

                        if event.faction == faction && slot == event.slot as u32 as i64 {
                            break 'pending;
                        }

                        index += 1;
                    }

                    let mut index = 0usize;

                    while index != ctx.counter_surge_events.len() {
                        let event = &ctx.counter_surge_events[index];

                        if event.faction == faction && slot == event.slot as u32 as i64 {
                            break 'pending;
                        }

                        index += 1;
                    }

                    finished = true;
                }

                let mut limit;

                if get_entity_state(ctx, faction, slot as i32)? == 4 {
                    if get_entity_frame(ctx, faction, slot as i32)? == 0 {
                        latch_battle_event(&mut ctx.battle_event_latch, faction, 1);
                    }

                    limit = 1;

                    if get_gudetama_soul(ctx, faction, slot as i32)? {
                        let button = get_entity_button(ctx, faction, slot as i32)?;
                        let anim = get_unit_anim(ctx, faction, button, 8)?
                            .ok_or(Fault::null_pointer())?;
                        let length = get_anim_len(anim)?;

                        limit = if length < 2 { 1 } else { length };
                    }

                    if get_soul_anim_type(ctx, faction, slot as i32)? >= 0 {
                        let key =
                            get_soul_anim_type(ctx, faction, slot as i32)?.wrapping_add(0x3e8);
                        let length = get_anim_len(std_map_int_maanim_subscript_2(
                            &mut ctx.effect_anims,
                            &key,
                        ))?;

                        if length > limit {
                            limit = length;
                        }
                    }
                } else {
                    limit = 1;

                    if get_entity_state(ctx, faction, slot as i32)? == 0x15 {
                        let delay = get_setting(&ctx.settings, b"battle_death_volcano_time", 0x1e)?;
                        let anim = ctx.death_surge_anims.get(faction as i64 as usize).ok_or(
                            Fault::index_out_of_range(faction as i64, 2),
                        )?;

                        limit = max_i32(delay, get_anim_len(anim)?);

                        if limit < 2 {
                            limit = 1;
                        }

                        let frame = get_entity_frame(ctx, faction, slot as i32)?;
                        let delay = get_setting(&ctx.settings, b"battle_death_volcano_time", 0x1e)?;

                        if frame == delay {
                            roll_procs(ctx, faction, slot as i32, 0)?;

                            let mut rolls = [0i32; 12];

                            for (index, roll) in rolls.iter_mut().enumerate() {
                                *roll = ctx.i32_at(AppContext::PROC_ROLLS + index * 4)?;
                            }

                            ctx.surge_events.push(SurgeEvent::default());

                            let event =
                                ctx.surge_events.last_mut().ok_or(Fault::index_out_of_range(0, 0))?;

                            event.faction = faction;
                            event.slot = slot as i32;
                            event.frame = 0;

                            let x = get_pos_x(ctx, faction, slot as i32)?;
                            let anchor = get_death_surge_anchor(ctx, faction, slot as i32)?;
                            let reach = abs_i32(get_death_surge_span(ctx, faction, slot as i32)?);
                            let drawn = call_rng(ctx, reach);
                            let span = get_death_surge_span(ctx, faction, slot as i32)?;
                            let event =
                                ctx.surge_events.last_mut().ok_or(Fault::index_out_of_range(0, 0))?;
                            let spread = if span > 0 {
                                drawn
                            } else {
                                drawn.wrapping_neg()
                            };

                            event.x = spread.wrapping_add(anchor.wrapping_add(x));

                            let level = get_death_surge_level(ctx, faction, slot as i32)?;
                            let event =
                                ctx.surge_events.last_mut().ok_or(Fault::index_out_of_range(0, 0))?;

                            event.level = level;
                            event.attack = 0;
                            event.proc_flags = [
                                (rolls[0] != 0) as u8,
                                (rolls[1] != 0) as u8,
                                (rolls[2] != 0) as u8,
                                (rolls[3] != 0) as u8,
                                (rolls[4] != 0) as u8,
                                (rolls[6] != 0) as u8,
                                (rolls[8] != 0) as u8,
                                (rolls[7] != 0) as u8,
                                (rolls[5] != 0) as u8,
                                (rolls[9] != 0) as u8,
                                (rolls[10] != 0) as u8,
                                (rolls[11] != 0) as u8,
                            ];
                            event.metal_killer_pct = 0;

                            let mini = get_death_surge_mini(ctx, faction, slot as i32)?;
                            let event =
                                ctx.surge_events.last_mut().ok_or(Fault::index_out_of_range(0, 0))?;

                            event.mini = mini != 0;
                            event.kind = 2;
                        }
                    }
                }

                if ctx.i32_at(entity.wrapping_add(Entity::FRAME))? < limit {
                    advance_animation_frame(ctx, faction, slot as i32)?;

                    break 'slot;
                }

                if !finished {
                    break 'slot;
                }

                ctx.set_i32_at(entity.wrapping_add(Entity::OCCUPANT), 0)?;
                ctx.set_block_at(entity.wrapping_add(Base::CANNON_WALL_OFFSET), [0u8; 8])?;
                ctx.set_i32_at(entity.wrapping_add(Base::CANNON_READY_VFX), 0)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 0x11 {
                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 5 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? != 0 {
                    if can_push_back(ctx, faction, slot as i32)? {
                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                        ctx.set_i32_at(entity.wrapping_add(Entity::POS_X), x.wrapping_add(-0x14))?;
                        keep_in_bound(ctx, faction, slot as i32)?;
                    }

                    let frame = ctx.i32_at(entity.wrapping_add(Entity::FRAME))? as i64;
                    let lift = *RECOIL_Y_ARC
                        .get(frame as usize)
                        .ok_or(Fault::index_out_of_range(frame, 12))?;
                    let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;

                    ctx.set_i32_at(entity.wrapping_add(Entity::POS_Y), y.wrapping_add(lift))?;

                    break 'slot;
                }

                if get_attacks_remaining(ctx, faction, slot as i32)? != 0 {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                } else {
                    no_more_attacks(ctx, faction, slot as i32)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 6 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? == 0 {
                    if get_attacks_remaining(ctx, faction, slot as i32)? != 0 {
                        set_entity_state(ctx, faction, slot as i32, 0)?;
                    } else {
                        no_more_attacks(ctx, faction, slot as i32)?;
                    }

                    break 'slot;
                }

                if !can_push_back(ctx, faction, slot as i32)? {
                    break 'slot;
                }

                let mut treasures: Vec<i32> = Vec::new();
                let red = if is_red(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0xc)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &red);

                let floating = if is_floating(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0xe)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &floating);

                let dark = if is_dark(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0xd)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &dark);

                let angel = if is_angel(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0xf)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &angel);

                let metal = if is_metal(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0x13)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &metal);

                let zombie = if is_zombie(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0x14)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &zombie);

                let alien = if is_alien(ctx, faction, slot as i32)? {
                    get_treasure_value(ctx, &ctx.treasure_store, 0x15)?
                } else {
                    0
                };

                std_vector_int_push_back(&mut treasures, &alien);

                let mut strongest = *treasures.first().ok_or(Fault::null_pointer())?;

                for &candidate in treasures.iter().skip(1) {
                    if strongest < candidate {
                        strongest = candidate;
                    }
                }

                let frame = ctx.i32_at(entity.wrapping_add(Entity::FRAME))?;
                let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0x11, -1)?;
                let sage_resist = get_sage_kb_resist_pct(ctx, faction, slot as i32)?;
                let remaining = 0xci32.wrapping_sub(frame);
                let cycles = operation::div_12(remaining as i64) as i32;
                let phase = remaining
                    .wrapping_sub(cycles.wrapping_shl(2).wrapping_mul(3))
                    .wrapping_mul(2)
                    .wrapping_mul(5);
                let boosted = (operation::div_1000(strongest.wrapping_mul(phase) as i64) as i32)
                    .wrapping_add(phase);
                let comboed =
                    operation::div_100(combo.wrapping_add(0x64).wrapping_mul(boosted) as i64)
                        as i32;
                let push = operation::div_neg_100(
                    0x64i32.wrapping_sub(sage_resist).wrapping_mul(comboed) as i64,
                ) as i32;
                let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                ctx.set_i32_at(entity.wrapping_add(Entity::POS_X), x.wrapping_add(push))?;
                get_cat_combo_bonus(ctx, &ctx.combo_store, 0x11, -1)?;
                get_sage_kb_resist_pct(ctx, faction, slot as i32)?;
                keep_in_bound(ctx, faction, slot as i32)?;

                break 'slot;
            } else {
                spawn_warp_tick(ctx, faction, slot as i32)?;
            }
        }

        slot += 1;

        if slot == 0x33 {
            break;
        }
    }

    for &bounty_slot in first_bounty_slots.iter() {
        set_first_bounty_pct(ctx, other, bounty_slot, 0)?;
    }

    let mut survivor = 1i32;

    loop {
        if slot_occupied(ctx, faction, survivor)? != 0 && get_hp(ctx, faction, survivor)? > 0 {
            let serial = get_spawn_serial(ctx, faction, survivor)?;

            if ctx.attackers_by_serial[credit].contains_key(&serial) {
                let serial = get_spawn_serial(ctx, faction, survivor)?;

                std_map_int_vector_erase(&mut ctx.attackers_by_serial[credit], &serial);
            }
        }

        survivor += 1;

        if survivor == 0x33 {
            break;
        }
    }

    Ok(())
}
