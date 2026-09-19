use crate::{Fault, operation};

use super::{
    AppContext, CANNON_SHOT_SPACING, CannonShot, Entity, cannon_attack_dispatch,
    cannon_shot_origin_x, get_anim_len, get_base_level, get_cannon_nonzombie_permille,
    get_cannon_shot_id, get_cannon_strike_width, get_cannon_strike_x, get_cannon_type,
    get_castle_anim_frame, get_castle_anim_state, get_entity_base_idx, get_hitbox_pos, get_pos_x,
    is_zombie, slot_occupied,
};

pub fn cannon_attack(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    let other = 1i32.wrapping_sub(faction);

    if get_cannon_type(ctx, faction)? == 0 || get_cannon_type(ctx, faction)? == 5 {
        let mut after = 0i32;
        let mut until = 0i32;

        if get_cannon_type(ctx, faction)? == 0 {
            after = 4;
            until = 8;
        } else if get_cannon_type(ctx, faction)? == 5 {
            let length = get_anim_len(&ctx.base_anims[1])?;

            until = length.wrapping_sub(4);
            after = length.wrapping_sub(8);
        }

        let shots = AppContext::CANNON_SHOTS.wrapping_add(
            (faction as i64 as usize).wrapping_mul(AppContext::CANNON_SHOTS_FACTION_STRIDE),
        );
        let mut shot = 0usize;

        while shot != 15 {
            let record = shots.wrapping_add(shot.wrapping_mul(AppContext::CANNON_SHOT_STRIDE));
            let timer = ctx.i32_at(record.wrapping_add(CannonShot::TIMER))?;

            if timer <= 0 {
                shot += 1;

                continue;
            }

            ctx.set_i32_at(
                record.wrapping_add(CannonShot::TIMER),
                timer.wrapping_sub(1),
            )?;

            if timer <= after || timer.wrapping_sub(1) > until {
                shot += 1;

                continue;
            }

            let mut slot = 1i32;

            while slot != 51 {
                'slot: {
                    if slot_occupied(ctx, other, slot)? != 2 {
                        break 'slot;
                    }

                    if faction == 1 {
                        let x = ctx
                            .i32_at(AppContext::entity_field(other, slot, Entity::POS_X))?
                            .wrapping_sub(ctx.i32_at(AppContext::entity_field(
                                other,
                                slot,
                                Entity::HITBOX_POS,
                            ))?);

                        if x > ctx.i32_at(record.wrapping_add(CannonShot::POS_X))? {
                            break 'slot;
                        }
                    } else {
                        if faction != 0 {
                            break 'slot;
                        }

                        if slot as i64 as u64 == get_entity_base_idx(ctx)? as u32 as u64 {
                            break 'slot;
                        }

                        let x = ctx
                            .i32_at(AppContext::entity_field(other, slot, Entity::POS_X))?
                            .wrapping_sub(ctx.i32_at(AppContext::entity_field(
                                other,
                                slot,
                                Entity::HITBOX_POS,
                            ))?);

                        if x < ctx.i32_at(record.wrapping_add(CannonShot::POS_X))? {
                            break 'slot;
                        }
                    }

                    if get_cannon_type(ctx, faction)? == 5
                        && !is_zombie(ctx, other, slot)?
                        && get_cannon_nonzombie_permille(ctx, faction)? == 0
                    {
                        break 'slot;
                    }

                    let shot_id = ctx.i32_at(record.wrapping_add(CannonShot::SHOT_ID))?;

                    cannon_attack_dispatch(ctx, faction, slot, shot_id)?;
                }

                slot += 1;
            }

            shot += 1;
        }

        return Ok(());
    }

    if get_castle_anim_state(ctx, faction)? == 3 || get_castle_anim_state(ctx, faction)? == 0xc {
        let origin = cannon_shot_origin_x(ctx, faction)?;
        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
        let reach = operation::div_32(
            get_castle_anim_frame(ctx, faction)?
                .wrapping_mul(step)
                .wrapping_mul(2)
                .wrapping_mul(5),
        );
        let limit = (if faction != 0 {
            reach
        } else {
            reach.wrapping_neg()
        })
        .wrapping_add(origin);
        let mut slot = 1i32;

        while slot != 51 {
            'slot: {
                if slot_occupied(ctx, other, slot)? != 2 {
                    break 'slot;
                }

                if faction == 0 && slot == get_entity_base_idx(ctx)? {
                    break 'slot;
                }

                let x =
                    get_pos_x(ctx, other, slot)?.wrapping_sub(get_hitbox_pos(ctx, other, slot)?);

                if x < limit {
                    break 'slot;
                }

                let shot_id = get_cannon_shot_id(ctx, faction)?;

                cannon_attack_dispatch(ctx, faction, slot, shot_id)?;
            }

            slot += 1;
        }

        return Ok(());
    }

    if (get_castle_anim_state(ctx, faction)? == 5
        && get_castle_anim_frame(ctx, faction)? >= 5
        && get_castle_anim_frame(ctx, faction)? < 0x10)
        || (get_castle_anim_state(ctx, faction)? == 0xb
            && get_castle_anim_frame(ctx, faction)? >= 0xa
            && get_castle_anim_frame(ctx, faction)? <= 0x14)
    {
        let mut slot = 1i32;

        while slot != 51 {
            'slot: {
                if slot_occupied(ctx, other, slot)? != 2 {
                    break 'slot;
                }

                if faction == 0 && slot == get_entity_base_idx(ctx)? {
                    break 'slot;
                }

                let pos = get_pos_x(ctx, other, slot)?;
                let hitbox = get_hitbox_pos(ctx, other, slot)?;
                let state = get_castle_anim_state(ctx, faction)?;
                let strike_x = get_cannon_strike_x(ctx, faction)?;
                let width = get_cannon_strike_width(ctx, 0)?;
                let offset = if state == 0xb {
                    operation::div_neg_10(width.wrapping_shl(3) as i64) as i32
                } else {
                    operation::div_2(width).wrapping_neg()
                };
                let x = pos.wrapping_sub(hitbox);
                let left = offset.wrapping_add(strike_x);
                let width = get_cannon_strike_width(ctx, 0)?;

                if left > x || x >= width.wrapping_add(left) {
                    break 'slot;
                }

                let shot_id = get_cannon_shot_id(ctx, faction)?;

                cannon_attack_dispatch(ctx, faction, slot, shot_id)?;
            }

            slot += 1;
        }

        return Ok(());
    }

    if get_castle_anim_state(ctx, faction)? == 8 && get_castle_anim_frame(ctx, faction)? >= 5 {
        let mut slot = 1i32;

        while slot != 51 {
            'slot: {
                if slot_occupied(ctx, other, slot)? != 2 {
                    break 'slot;
                }

                if faction == 0 && slot == get_entity_base_idx(ctx)? {
                    break 'slot;
                }

                let x =
                    get_pos_x(ctx, other, slot)?.wrapping_sub(get_hitbox_pos(ctx, other, slot)?);
                let left = get_cannon_strike_x(ctx, faction)?
                    .wrapping_sub(operation::div_2(get_cannon_strike_width(ctx, 0)?));
                let strike_x = get_cannon_strike_x(ctx, faction)?;
                let width = get_cannon_strike_width(ctx, 0)?;

                if left > x || x >= operation::div_2(width).wrapping_add(strike_x) {
                    break 'slot;
                }

                let shot_id = get_cannon_shot_id(ctx, faction)?;

                cannon_attack_dispatch(ctx, faction, slot, shot_id)?;
            }

            slot += 1;
        }
    }

    Ok(())
}
