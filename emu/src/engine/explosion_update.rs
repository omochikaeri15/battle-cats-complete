use crate::{Fault, ops};

use super::{
    AppContext, EXPLOSION_DAMAGE_DEFAULTS, EXPLOSION_WIDTH_DEFAULTS, Entity, explosion_attack,
    get_pos_x, get_setting, is_touchable, min_i32, play_sound_in_battle, slot_occupied,
};

pub fn explosion_update(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut event_index = 0i32;

    while (event_index as i64 as usize) < ctx.explosion_events.len() {
        let at = event_index as i64 as usize;
        let missing = Fault::index_out_of_range(event_index as i64, ctx.explosion_events.len() as i64);
        let event = ctx.explosion_events.get_mut(at).ok_or(missing.clone())?;
        let faction = event.faction;
        let frame = event.frame.wrapping_add(1);

        event.frame = frame;

        let interval = get_setting(&ctx.settings, b"battle_explosion_frame4", 0xf)?;
        let rings = min_i32(
            ops::idiv(frame, interval)
                .ok_or(Fault::divide(interval as i64))?
                .wrapping_add(1),
            3,
        );

        if rings > 0 {
            let other = 1i32.wrapping_sub(faction);
            let mut ring = 0usize;

            while ring != rings as u32 as usize {
                let number = (ring as u64).wrapping_add(1).to_string();
                let default_width =
                    *EXPLOSION_WIDTH_DEFAULTS
                        .get(ring)
                        .ok_or(Fault::index_out_of_range(ring as i64, 3))?;
                let width = get_setting(
                    &ctx.settings,
                    &[b"battle_explosion_width".as_slice(), number.as_bytes()].concat(),
                    default_width,
                )?;
                let mut spread = 0i32;

                match ring {
                    2 => {
                        let width1 = get_setting(&ctx.settings, b"battle_explosion_width1", 0x320)?;
                        let width2 = get_setting(&ctx.settings, b"battle_explosion_width2", 0x1f4)?;
                        let width3 = get_setting(&ctx.settings, b"battle_explosion_width3", 0x190)?;

                        spread = width1
                            .wrapping_add(width2.wrapping_mul(2))
                            .wrapping_add(width3);
                    }
                    1 => {
                        let width1 = get_setting(&ctx.settings, b"battle_explosion_width1", 0x320)?;

                        spread = get_setting(&ctx.settings, b"battle_explosion_width2", 0x1f4)?;
                        spread = spread.wrapping_add(width1);
                    }
                    _ => {}
                }

                let frame = ctx.explosion_events.get(at).ok_or(missing.clone())?.frame;
                let interval = get_setting(&ctx.settings, b"battle_explosion_frame4", 0xf)?;
                let start = get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)?;
                let elapsed = frame.wrapping_sub(interval.wrapping_mul(ring as i32));

                if elapsed == start {
                    play_sound_in_battle(ctx, (ring as i32).wrapping_add(0xa7))?;
                }

                let active =
                    if elapsed >= get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)? {
                        let start = get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)?;

                        elapsed
                            < get_setting(&ctx.settings, b"battle_explosion_frame2", 0x14)?
                                .wrapping_add(start)
                    } else {
                        false
                    };

                if !active {
                    ring += 1;

                    continue;
                }

                let left = ops::div_2(spread).wrapping_neg();
                let half = ops::div_2(width);
                let mut slot = 1i32;

                while slot != 51 {
                    if slot_occupied(ctx, other, slot)? == 0 {
                        slot += 1;

                        continue;
                    }

                    let owner = ctx.explosion_events.get(at).ok_or(missing.clone())?.slot;

                    if !is_touchable(ctx, other, slot, owner)? {
                        slot += 1;

                        continue;
                    }

                    let x = if faction == 1 {
                        get_pos_x(ctx, 0, slot)?.wrapping_add(
                            ctx.i32_at(AppContext::entity_field(0, slot, Entity::HITBOX_POS))?,
                        )
                    } else {
                        get_pos_x(ctx, other, slot)?.wrapping_sub(
                            ctx.i32_at(AppContext::entity_field(other, slot, Entity::HITBOX_POS))?,
                        )
                    };
                    let mut pass = 0i32;
                    let mut first = true;

                    loop {
                        'pass: {
                            let event = ctx.explosion_events.get(at).ok_or(missing.clone())?;
                            let hit = ring.wrapping_add(if first { 0 } else { 2 });

                            if event.hits.contains_key(&slot) {
                                let flags = event.hits.get(&slot).ok_or(Fault::key_not_found(slot as i64))?;

                                if *flags.get(hit).ok_or(Fault::out_of_range())? != 0 {
                                    break 'pass;
                                }
                            }

                            let center = pass
                                .wrapping_mul(spread)
                                .wrapping_add(left)
                                .wrapping_add(event.x);

                            if x < center.wrapping_sub(half) || x >= center.wrapping_add(half) {
                                break 'pass;
                            }

                            let attack = event.attack;
                            let default_damage = *EXPLOSION_DAMAGE_DEFAULTS.get(ring).ok_or(
                                Fault::index_out_of_range(ring as i64, 3),
                            )?;
                            let dmg_scale = get_setting(
                                &ctx.settings,
                                &[b"battle_explosion_damage".as_slice(), number.as_bytes()]
                                    .concat(),
                                default_damage,
                            )?;

                            explosion_attack(ctx, event_index, slot, attack, dmg_scale)?;

                            let flags = ctx
                                .explosion_events
                                .get_mut(at)
                                .ok_or(missing.clone())?
                                .hits
                                .entry(slot)
                                .or_insert([0; 5]);

                            *flags.get_mut(hit).ok_or(Fault::index_out_of_range(hit as i64, 5))? = 1;
                        }

                        let again = ring != 0 && first;

                        pass = 1;
                        first = false;

                        if !again {
                            break;
                        }
                    }

                    slot += 1;
                }

                ring += 1;
            }
        }

        let frame = ctx.explosion_events.get(at).ok_or(missing.clone())?.frame;
        let interval = get_setting(&ctx.settings, b"battle_explosion_frame4", 0xf)?;
        let start = get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)?;
        let burn = get_setting(&ctx.settings, b"battle_explosion_frame2", 0x14)?;
        let fade = get_setting(&ctx.settings, b"battle_explosion_frame3", 0xa)?;

        if frame
            >= start
                .wrapping_add(interval.wrapping_mul(2))
                .wrapping_add(burn)
                .wrapping_add(fade)
        {
            ctx.explosion_events.remove(at);
            event_index = event_index.wrapping_sub(1);
        }

        event_index = event_index.wrapping_add(1);
    }

    Ok(())
}
