use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{
    AppContext, Base, CANNON_SHOT_SPACING, CannonShot, Debris, Entity, KNOCKBACK_Y_ARC,
    RECOIL_Y_ARC, WaveRecord, abs_i32, add_castle_anim_frame, add_money, add_pos_x,
    advance_animation_frame, call_rng, can_push_back, cannon_shot_origin_x, check_collision,
    get_anim_len, get_attacks_remaining, get_base_hp, get_base_level, get_battle_status,
    get_cannon_charge_orb, get_cannon_recharge, get_cannon_shot_id, get_cannon_type,
    get_cash_back_pct, get_castle_anim_frame, get_castle_anim_state, get_conjure_unit_id,
    get_curse_timer, get_death_surge_anchor, get_death_surge_level, get_death_surge_mini,
    get_death_surge_span, get_effective_deploy_cost, get_entity_button, get_entity_frame,
    get_entity_state, get_freeze_timer, get_gudetama_soul, get_knockback_resist_pct,
    get_metal_killer_pct, get_paid_cost, get_pos_x, get_sage_kb_resist_pct, get_setting,
    get_slot_unit_id, get_slow_timer, get_soul_anim_type, get_speed, get_unit_anim,
    get_weaken_timer, keep_in_bound, latch_battle_event, max_i32, min_i32, no_more_attacks,
    play_sound, roll_procs, set_castle_anim_frame, set_castle_anim_state, set_crit_vfx,
    set_drain_pct, set_entity_frame, set_entity_state, set_frame_damage, set_metal_killer_vfx,
    set_prev_curse_timer, set_prev_freeze_timer, set_prev_slow_timer, set_prev_weaken_timer,
    set_proc_badge, set_savage_blow_vfx, set_score_hit_mask, set_toxic_vfx, slot_occupied,
    sound_manager, spawn_warp_tick, std_map_int_maanim_subscript_2,
};

#[derive(Default)]
pub struct SurgeEvent {
    pub faction: i32,
    pub slot: i32,
    pub frame: i32,
    pub x: i32,
    pub level: i32,
    pub attack: i32,
    pub proc_flags: [u8; 12],
    pub metal_killer_pct: i32,
    pub mini: bool,
    pub kind: i32,
    pub hit_ticks: BTreeMap<i32, i32>,
}

pub fn cat_update(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    ctx.metal_killer_map.clear();

    let knockback_step = if faction == 0 { 0x3c } else { -0x3c };
    let recoil_step = if faction == 0 { 0x14 } else { -0x14 };
    let shot_offset = if faction == 0 { faction } else { -0x73a };
    let wallet = AppContext::faction_flags(faction);
    let shots = AppContext::CANNON_SHOTS.wrapping_add(
        (faction as i64 as usize).wrapping_mul(AppContext::CANNON_SHOTS_FACTION_STRIDE),
    );
    let base = AppContext::entity_field(faction, 0, 0);

    let mut slot = 0i64;

    'slots: loop {
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

            if slot == 0 {
                'castle: {
                    let reset_to_idle;

                    if get_castle_anim_state(ctx, faction)? == 1 {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        if get_castle_anim_frame(ctx, faction)? >= 0x11 {
                            set_castle_anim_state(ctx, faction, 2)?;
                            set_castle_anim_frame(ctx, faction, 0)?;
                            sound_manager(ctx)?.stop_audio(0x19);
                        }

                        break 'castle;
                    } else if get_castle_anim_state(ctx, faction)? == 2
                        || get_castle_anim_state(ctx, faction)? == 0xa
                    {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        if get_castle_anim_frame(ctx, faction)? & 3 != 0 {
                            break 'castle;
                        }

                        let mut shot = 0i64;

                        loop {
                            let timer = shots.wrapping_add(
                                (shot as usize).wrapping_mul(AppContext::CANNON_SHOT_STRIDE),
                            );

                            if ctx.i32_at(timer.wrapping_add(CannonShot::TIMER))? == 0 {
                                break;
                            }

                            shot += 1;

                            if shot == 0xf {
                                break 'castle;
                            }
                        }

                        let record = shots.wrapping_add(
                            (shot as usize).wrapping_mul(AppContext::CANNON_SHOT_STRIDE),
                        );

                        if get_battle_status(ctx)? == 0 {
                            let mut sound_id = 0x1a;
                            let mut firing = get_castle_anim_state(ctx, faction)? == 2;

                            if !firing {
                                sound_id = 0x55;
                                firing = get_castle_anim_state(ctx, faction)? == 0xa;
                            }

                            if firing {
                                play_sound(sound_manager(ctx)?, sound_id, None);
                            }
                        }

                        if get_cannon_type(ctx, faction)? == 0 {
                            let length = get_anim_len(&ctx.wave_anim)?.wrapping_sub(1);

                            ctx.set_i32_at(record.wrapping_add(CannonShot::TIMER), length)?;
                        } else if get_cannon_type(ctx, faction)? == 5 {
                            let length = get_anim_len(&ctx.base_anims[1])?;

                            ctx.set_i32_at(record.wrapping_add(CannonShot::TIMER), length)?;
                        }

                        let origin = cannon_shot_origin_x(ctx, faction)?;
                        let frame = get_castle_anim_frame(ctx, faction)?;
                        let volley = (if frame >= 0 {
                            frame
                        } else {
                            frame.wrapping_add(3)
                        }) >> 2;
                        let spread = volley
                            .wrapping_mul(CANNON_SHOT_SPACING)
                            .wrapping_mul(2)
                            .wrapping_mul(5);
                        let offset = if faction != 0 {
                            spread
                        } else {
                            spread.wrapping_neg()
                        };
                        let x = origin.wrapping_add(shot_offset).wrapping_add(offset);

                        ctx.set_i32_at(record.wrapping_add(CannonShot::POS_X), x)?;

                        let shot_id = get_cannon_shot_id(ctx, faction)?;

                        ctx.set_i32_at(record.wrapping_add(CannonShot::SHOT_ID), shot_id)?;

                        let frame = get_castle_anim_frame(ctx, faction)?;
                        let volley = (if frame >= 0 {
                            frame
                        } else {
                            frame.wrapping_add(3)
                        }) >> 2;

                        if volley >= get_base_level(ctx, faction)? {
                            set_castle_anim_state(ctx, faction, 0)?;
                            set_castle_anim_frame(ctx, faction, 0)?;
                        }

                        break 'castle;
                    } else if get_castle_anim_state(ctx, faction)? == 3
                        || get_castle_anim_state(ctx, faction)? == 0xc
                    {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        reset_to_idle = get_castle_anim_frame(ctx, faction)? >= 0x21;
                    } else if get_castle_anim_state(ctx, faction)? == 4 {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        let frame = get_castle_anim_frame(ctx, faction)?;

                        if frame > get_anim_len(&ctx.base_anims[1])? {
                            set_castle_anim_state(ctx, 0, 5)?;
                            set_castle_anim_frame(ctx, 0, 0)?;
                            play_sound(sound_manager(ctx)?, 0x24, None);
                        }

                        break 'castle;
                    } else if get_castle_anim_state(ctx, faction)? == 7 {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        let frame = get_castle_anim_frame(ctx, faction)?;

                        if frame > get_anim_len(&ctx.base_anims[1])? {
                            set_castle_anim_state(ctx, 0, 8)?;
                            set_castle_anim_frame(ctx, 0, 0)?;
                            play_sound(sound_manager(ctx)?, 0x53, None);
                        }

                        break 'castle;
                    } else if get_castle_anim_state(ctx, faction)? == 5
                        || get_castle_anim_state(ctx, faction)? == 8
                    {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        let frame = get_castle_anim_frame(ctx, faction)?;

                        reset_to_idle = frame > get_anim_len(&ctx.base_anims[1])?;
                    } else if get_castle_anim_state(ctx, faction)? == 6 {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        let frame = get_castle_anim_frame(ctx, faction)?;

                        reset_to_idle = frame > get_anim_len(&ctx.base_anims[0])?;
                    } else if get_castle_anim_state(ctx, faction)? == 9 {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        if get_castle_anim_frame(ctx, faction)? >= 0x11 {
                            set_castle_anim_state(ctx, faction, 0xa)?;
                            set_castle_anim_frame(ctx, faction, 0)?;
                            sound_manager(ctx)?.stop_audio(0x54);
                        }

                        break 'castle;
                    } else if get_castle_anim_state(ctx, faction)? == 0xb {
                        add_castle_anim_frame(ctx, faction, 1)?;

                        if get_castle_anim_frame(ctx, faction)? == 5 {
                            play_sound(sound_manager(ctx)?, 0x56, None);

                            break 'castle;
                        }

                        let frame = get_castle_anim_frame(ctx, faction)?.wrapping_add(-0xa);

                        reset_to_idle = frame > get_anim_len(&ctx.base_anims[2])?;
                    } else {
                        break 'castle;
                    }

                    if reset_to_idle {
                        set_castle_anim_state(ctx, 0, 0)?;
                        set_castle_anim_frame(ctx, 0, 0)?;
                    }
                }

                if get_base_hp(ctx, faction)? <= 0 {
                    ctx.set_i32_at(base.wrapping_add(Base::STATE), 2)?;

                    let counter = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;
                    let ring = counter
                        .wrapping_sub((ops::div_5(counter as i64) as i32).wrapping_mul(5))
                        .wrapping_add(0x32);
                    let record = AppContext::CAT_DEBRIS.wrapping_add(
                        (ring as u32 as usize).wrapping_mul(AppContext::DEBRIS_STRIDE),
                    );

                    ctx.set_i32_at(record.wrapping_add(Debris::TIMER), 0xc)?;

                    let x = ctx.i32_at(base.wrapping_add(Entity::POS_X))?;
                    let scatter = call_rng(ctx, 0xf1);

                    ctx.set_i32_at(
                        record.wrapping_add(Debris::POS_X),
                        x.wrapping_add(scatter.wrapping_mul(5).wrapping_mul(2))
                            .wrapping_add(-0x43f),
                    )?;

                    let y = ctx.i32_at(base.wrapping_add(Entity::POS_Y))?;
                    let scatter = call_rng(ctx, 0x143);

                    ctx.set_i32_at(
                        record.wrapping_add(Debris::POS_Y),
                        scatter
                            .wrapping_mul(2)
                            .wrapping_mul(5)
                            .wrapping_neg()
                            .wrapping_add(y)
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

            let refund_on_death;

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

                let step = if faction != 0 {
                    speed
                } else {
                    speed.wrapping_neg()
                };

                add_pos_x(ctx, faction, slot as i32, step)?;
                advance_animation_frame(ctx, faction, slot as i32)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 3 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? != 0 {
                    if can_push_back(ctx, faction, slot as i32)? {
                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                        ctx.set_i32_at(
                            entity.wrapping_add(Entity::POS_X),
                            x.wrapping_add(knockback_step),
                        )?;
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

                if ctx.i32_at(entity.wrapping_add(Entity::HP))? != 0 {
                    break 'slot;
                }

                refund_on_death = true;
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
                            && ctx.i32_at(first.wrapping_add(WaveRecord::IN_USE))? == 1
                        {
                            finished = false;

                            break 'pending;
                        }

                        if slot
                            == ctx.i32_at(second.wrapping_add(WaveRecord::OWNER_SLOT))? as u32
                                as i64
                            && ctx.i32_at(second.wrapping_add(WaveRecord::IN_USE))? == 1
                        {
                            finished = false;

                            break 'pending;
                        }

                        pair += 1;

                        if pair == 100 {
                            break;
                        }
                    }

                    finished = true;

                    let mut index = 0usize;

                    while index != ctx.surge_events.len() {
                        let event = &ctx.surge_events[index];

                        if event.faction == faction && slot == event.slot as u32 as i64 {
                            finished = false;

                            break 'pending;
                        }

                        index += 1;
                    }
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

                            let metal_killer_pct = get_metal_killer_pct(ctx, faction, slot as i32)?;

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
                            let spread = if span <= 0 {
                                drawn
                            } else {
                                drawn.wrapping_neg()
                            };

                            event.x = x.wrapping_sub(anchor).wrapping_add(spread);

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
                            event.metal_killer_pct = metal_killer_pct;

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

                'conjure: {
                    if get_conjure_unit_id(ctx, faction, slot as i32)? >= 0 {
                        let mut other = 0i64;

                        loop {
                            if slot != other
                                && slot_occupied(ctx, faction, other as i32)? != 0
                                && get_slot_unit_id(ctx, faction, other as i32)?
                                    == get_slot_unit_id(ctx, faction, slot as i32)?
                                && get_entity_state(ctx, faction, other as i32)? != 4
                            {
                                break 'conjure;
                            }

                            other += 1;

                            if other as i32 == 0x33 {
                                break;
                            }
                        }

                        let button = get_entity_button(ctx, faction, slot as i32)? as i64;

                        ctx.set_i32_at(
                            wallet
                                .wrapping_add(AppContext::WALLET_CONJURE_READY)
                                .wrapping_add((button * 4) as usize),
                            0,
                        )?;
                    }
                }

                if faction == 0
                    && get_cannon_charge_orb(ctx, 0, slot as i32)? > 0
                    && ctx.i32_at(base.wrapping_add(Base::CANNON_COUNTDOWN))? > 0
                {
                    let recharge = get_cannon_recharge(ctx, 0)?;
                    let cut = ops::div_1000(
                        get_cannon_charge_orb(ctx, 0, slot as i32)?.wrapping_mul(recharge) as i64,
                    ) as i32;

                    get_cannon_recharge(ctx, 0)?;
                    get_cannon_charge_orb(ctx, 0, slot as i32)?;
                    max_i32(
                        ctx.i32_at(base.wrapping_add(Base::CANNON_COUNTDOWN))?
                            .wrapping_sub(cut),
                        1,
                    );

                    let mut countdown = ctx
                        .i32_at(base.wrapping_add(Base::CANNON_COUNTDOWN))?
                        .wrapping_sub(cut);

                    if countdown < 2 {
                        countdown = 1;
                    }

                    ctx.set_i32_at(base.wrapping_add(Base::CANNON_COUNTDOWN), countdown)?;
                }

                ctx.set_i32_at(entity.wrapping_add(Entity::OCCUPANT), 0)?;
                ctx.set_block_at(entity.wrapping_add(Base::CANNON_WALL_OFFSET), [0u8; 8])?;
                ctx.set_i32_at(entity.wrapping_add(Base::CANNON_READY_VFX), 0)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 5 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? != 0 {
                    if can_push_back(ctx, faction, slot as i32)? {
                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                        ctx.set_i32_at(
                            entity.wrapping_add(Entity::POS_X),
                            x.wrapping_add(recoil_step),
                        )?;
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

                if get_attacks_remaining(ctx, faction, slot as i32)? == 0 {
                    no_more_attacks(ctx, faction, slot as i32)?;
                } else {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                }

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 6 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? == 0 {
                    if get_attacks_remaining(ctx, faction, slot as i32)? == 0 {
                        no_more_attacks(ctx, faction, slot as i32)?;
                    } else {
                        set_entity_state(ctx, faction, slot as i32, 0)?;
                    }

                    break 'slot;
                }

                if !can_push_back(ctx, faction, slot as i32)? {
                    break 'slot;
                }

                let remaining =
                    0xci32.wrapping_sub(ctx.i32_at(entity.wrapping_add(Entity::FRAME))?);
                let cycles = ops::div_12(remaining as i64) as i32;
                let phase = remaining
                    .wrapping_sub(cycles.wrapping_shl(2).wrapping_mul(3))
                    .wrapping_mul(2)
                    .wrapping_mul(5);
                let push = if faction == 0 {
                    phase
                } else {
                    phase.wrapping_neg()
                };
                let resisted = 0x64i32
                    .wrapping_sub(get_knockback_resist_pct(ctx, faction, slot as i32)?)
                    .wrapping_mul(push);
                let push = ops::div_100(resisted as i64) as i32;
                let resisted = 0x64i32
                    .wrapping_sub(get_sage_kb_resist_pct(ctx, faction, slot as i32)?)
                    .wrapping_mul(push);
                let push = ops::div_100(resisted as i64) as i32;
                let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                ctx.set_i32_at(entity.wrapping_add(Entity::POS_X), x.wrapping_add(push))?;
                keep_in_bound(ctx, faction, slot as i32)?;
                get_knockback_resist_pct(ctx, faction, slot as i32)?;
                get_sage_kb_resist_pct(ctx, faction, slot as i32)?;

                break 'slot;
            } else if get_entity_state(ctx, faction, slot as i32)? == 7 {
                advance_animation_frame(ctx, faction, slot as i32)?;

                if get_entity_frame(ctx, faction, slot as i32)? != 0 {
                    if can_push_back(ctx, faction, slot as i32)? {
                        let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;

                        ctx.set_i32_at(
                            entity.wrapping_add(Entity::POS_X),
                            x.wrapping_add(knockback_step),
                        )?;
                        keep_in_bound(ctx, faction, slot as i32)?;
                    }

                    let frame = ctx.i32_at(entity.wrapping_add(Entity::FRAME))?;
                    let half = (((frame as u32) >> 0x1f) as i32).wrapping_add(frame) >> 1;
                    let lift = *KNOCKBACK_Y_ARC.get(half as i64 as usize).ok_or(
                        Fault::index_out_of_range(half as i64, 24),
                    )?;
                    let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;

                    ctx.set_i32_at(entity.wrapping_add(Entity::POS_Y), y.wrapping_add(lift))?;

                    break 'slot;
                }

                if get_attacks_remaining(ctx, faction, slot as i32)? == 0 {
                    no_more_attacks(ctx, faction, slot as i32)?;
                } else {
                    set_entity_state(ctx, faction, slot as i32, 0)?;
                }

                if ctx.i32_at(entity.wrapping_add(Entity::HP))? != 0 {
                    break 'slot;
                }

                refund_on_death = false;
            } else {
                spawn_warp_tick(ctx, faction, slot as i32)?;

                break 'slot;
            }

            if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 || get_battle_status(ctx)? != 4 {
                play_sound(sound_manager(ctx)?, 0x18, None);
            }

            set_entity_state(ctx, faction, slot as i32, 4)?;
            set_entity_frame(ctx, faction, slot as i32, 0)?;

            if refund_on_death && get_cash_back_pct(ctx, faction, slot as i32)? > 0 {
                let mut cost = get_paid_cost(ctx, faction, slot as i32)?;

                if cost == 0 {
                    let button = get_entity_button(ctx, faction, slot as i32)?;

                    cost = get_effective_deploy_cost(ctx, faction, button)?;
                }

                let refund = ops::div_100(
                    get_cash_back_pct(ctx, faction, slot as i32)?.wrapping_mul(cost) as i64,
                ) as i32;

                add_money(ctx, wallet, max_i32(refund, 1))?;
                get_cash_back_pct(ctx, faction, slot as i32)?;
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
        }

        slot += 1;

        if slot == 0x33 {
            break 'slots;
        }
    }

    Ok(())
}
