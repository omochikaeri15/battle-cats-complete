use crate::{Fault, operation};

use super::{
    AppContext, Entity, SURGE_TIMING, base_shake_start, get_pos_x, is_touchable,
    play_sound_in_battle, slot_occupied, surge_attack,
};

pub fn surge_update(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut index = 0i32;

    while (index as i64 as usize) < ctx.surge_events.len() {
        let at = index as i64 as usize;
        let missing = Fault::index_out_of_range(index as i64, ctx.surge_events.len() as i64);
        let event = ctx.surge_events.get_mut(at).ok_or(missing.clone())?;
        let before = event.frame;

        event.frame = before.wrapping_add(1);

        let elapsed = event.frame.wrapping_sub(SURGE_TIMING[1]);

        if elapsed < 0 {
            if before == 0 {
                play_sound_in_battle(ctx, 0x6f)?;
            }

            index = index.wrapping_add(1);

            continue;
        }

        let interval = SURGE_TIMING[2];
        let lifetime = event.level.wrapping_mul(interval);

        if elapsed >= lifetime {
            if elapsed >= lifetime.wrapping_add(SURGE_TIMING[3]) {
                ctx.surge_events.remove(at);
                index = index.wrapping_sub(1);
            }

            index = index.wrapping_add(1);

            continue;
        }

        if operation::irem(elapsed, interval).ok_or(Fault::divide(interval as i64))? == 0 {
            base_shake_start(ctx, 2);
            play_sound_in_battle(ctx, 0x70)?;
        }

        let tick = operation::idiv(elapsed, SURGE_TIMING[2])
            .ok_or(Fault::divide(SURGE_TIMING[2] as i64))?;
        let faction = ctx.surge_events.get(at).ok_or(missing.clone())?.faction;
        let other = 1i32.wrapping_sub(faction);
        let mut slot = 1i32;

        while slot != 51 {
            if slot_occupied(ctx, other, slot)? == 0 {
                slot += 1;

                continue;
            }

            let owner = ctx.surge_events.get(at).ok_or(missing.clone())?.slot;

            if !is_touchable(ctx, other, slot, owner)? {
                slot += 1;

                continue;
            }

            let hit_ticks = &ctx.surge_events.get(at).ok_or(missing.clone())?.hit_ticks;

            if hit_ticks.contains_key(&slot)
                && *hit_ticks.get(&slot).ok_or(Fault::key_not_found(slot as i64))? == tick
            {
                slot += 1;

                continue;
            }

            let x = if faction == 1 {
                get_pos_x(ctx, 0, slot)?
                    .wrapping_add(ctx.i32_at(AppContext::entity_field(
                        0,
                        slot,
                        Entity::HITBOX_POS,
                    ))?)
                    .wrapping_sub(SURGE_TIMING[4])
            } else {
                get_pos_x(ctx, other, slot)?
                    .wrapping_sub(ctx.i32_at(AppContext::entity_field(
                        other,
                        slot,
                        Entity::HITBOX_POS,
                    ))?)
                    .wrapping_add(SURGE_TIMING[4])
            };
            let event = ctx.surge_events.get(at).ok_or(missing.clone())?;
            let half = operation::div_2(SURGE_TIMING[0]);

            if x < event.x.wrapping_sub(half) || x >= half.wrapping_add(event.x) {
                slot += 1;

                continue;
            }

            let attack = event.attack;

            surge_attack(ctx, index, slot, attack)?;
            *ctx.surge_events
                .get_mut(at)
                .ok_or(missing.clone())?
                .hit_ticks
                .entry(slot)
                .or_insert(0) = tick;
            slot += 1;
        }

        index = index.wrapping_add(1);
    }

    Ok(())
}
