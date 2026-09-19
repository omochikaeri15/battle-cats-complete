use crate::{operation, Fault};

use super::{
    atan2_deg, cos_deg, get_base_pos_x, get_base_pos_y, get_battle_status, get_design_height2, get_entity_base_idx, get_powerup, is_touchable, sin_deg, AppContext, Entity,
};

pub fn sniper_update(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::SNIPER_TARGET, 0)?;
    ctx.set_i32_at(AppContext::SCRATCH_0, 0)?;

    if !get_powerup(ctx, 5)? {
        ctx.set_block_at::<1>(AppContext::SNIPER_CASINGS_LIVE, [0])?;
        ctx.set_block_at::<1>(AppContext::SNIPER_FIRING, [0])?;

        let mut strike = 0usize;

        while strike != 50 {
            if ctx.u8_at(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike))? != 0 {
                ctx.set_block_at::<1>(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike), [0])?;
            }

            strike += 1;
        }

        return Ok(());
    }

    if get_battle_status(ctx)? == 0 {
        let mut slot = 1i64;

        while slot != 51 {
            'slot: {
                if ctx.i32_at(AppContext::entity_field(1, slot as i32, Entity::OCCUPANT))? == 0 {
                    break 'slot;
                }

                if !is_touchable(ctx, 1, slot as i32, -1)? {
                    break 'slot;
                }

                if slot as u64 == get_entity_base_idx(ctx)? as u32 as u64 {
                    break 'slot;
                }

                let x = ctx.i32_at(AppContext::entity_field(1, slot as i32, Entity::POS_X))?;

                if x < ctx.i32_at(AppContext::SCRATCH_0)? {
                    break 'slot;
                }

                ctx.set_i32_at(AppContext::SCRATCH_0, x)?;
                ctx.set_i32_at(AppContext::SNIPER_TARGET, slot as i32)?;
            }

            slot += 1;
        }

        let gun_x = operation::div_10(get_base_pos_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?)).wrapping_add(0xcb);

        ctx.set_i32_at(AppContext::SCRATCH_1, gun_x)?;

        let rest_y = operation::div_10(get_base_pos_y(ctx, 0)?).wrapping_add(-0x171) as f32;
        let gun_y = operation::cvttss2si(sin_deg(ctx.f32_at(AppContext::SNIPER_BOB_ANGLE)?) * 10.0 + rest_y);

        ctx.set_i32_at(AppContext::SCRATCH_2, gun_y)?;
    }

    let target = ctx.i32_at(AppContext::SNIPER_TARGET)?;
    let mut goal = 0.0f32;

    if target > 0 {
        let pos_y = ctx.i32_at(AppContext::entity_field(1, target, Entity::POS_Y))?;
        let z_layer = ctx.i32_at(AppContext::entity_field(1, target, Entity::Z_LAYER))?;
        let rise = (operation::div_neg_10(pos_y as i64) as i32).wrapping_add(ctx.i32_at(AppContext::SCRATCH_2)?).wrapping_sub(z_layer.wrapping_shl(2)).wrapping_add(0x3a) as f32;
        let pos_x = ctx.i32_at(AppContext::entity_field(1, target, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
        let run = (operation::div_neg_10(pos_x as i64) as i32).wrapping_add(ctx.i32_at(AppContext::SCRATCH_1)?) as f32;
        let turn = 360.0f32 - atan2_deg(rise, run);
        let mirrored = -turn;

        goal = if mirrored >= turn { 0.0 } else { mirrored };
    }

    ctx.set_f32_at(AppContext::SNIPER_AIM_GOAL, goal)?;
    ctx.set_i32_at(AppContext::SCRATCH_0, 0)?;

    let mut aim = ctx.f32_at(AppContext::SNIPER_AIM_ANGLE)?;
    let arrived;

    if aim >= goal {
        aim += -1.0;
        ctx.set_f32_at(AppContext::SNIPER_AIM_ANGLE, aim)?;
        arrived = goal >= aim;
    } else if goal >= aim {
        aim += 1.0;
        ctx.set_f32_at(AppContext::SNIPER_AIM_ANGLE, aim)?;
        arrived = aim >= goal;
    } else {
        arrived = false;
    }

    if arrived {
        ctx.set_f32_at(AppContext::SNIPER_AIM_ANGLE, goal)?;
        ctx.set_i32_at(AppContext::SCRATCH_0, 1)?;
    }

    if ctx.i32_at(AppContext::SNIPER_TARGET)? == 0 {
        ctx.set_i32_at(AppContext::SCRATCH_0, 0)?;
    }

    let muzzle_x = get_base_pos_x(ctx, 0)?.wrapping_add(0x6ea);

    ctx.set_i32_at(AppContext::SCRATCH_1, muzzle_x)?;

    let rest_y = get_base_pos_y(ctx, 0)?.wrapping_add(-0xf76) as f32;
    let muzzle_y = operation::cvttss2si(sin_deg(ctx.f32_at(AppContext::SNIPER_BOB_ANGLE)?) * 10.0 * 10.0 + rest_y);

    ctx.set_i32_at(AppContext::SCRATCH_2, muzzle_y)?;

    let charge = ctx.i32_at(AppContext::SNIPER_CHARGE)?;
    let charge = if charge < 0x12b { charge.wrapping_add(1) } else { 0x12c };

    ctx.set_i32_at(AppContext::SNIPER_CHARGE, charge)?;

    let firing = ctx.u8_at(AppContext::SNIPER_FIRING)?;

    'sequence: {
        if ctx.i32_at(AppContext::SCRATCH_0)? == 1 && charge >= 0x12c {
            if firing == 0 {
                ctx.set_i32_at(AppContext::SNIPER_CHARGE, 0)?;
                ctx.set_block_at::<1>(AppContext::SNIPER_FIRING, [1])?;
                ctx.set_i32_at(AppContext::SNIPER_FIRE_FRAME, 0)?;
            }
        } else if firing == 0 {
            break 'sequence;
        }

        let before = ctx.i32_at(AppContext::SNIPER_FIRE_FRAME)?;
        let mut frame = before.wrapping_add(1);

        ctx.set_i32_at(AppContext::SNIPER_FIRE_FRAME, frame)?;

        match before {
            1 | 3 => {
                ctx.set_i32_at(AppContext::SNIPER_RECOIL, -6)?;

                break 'sequence;
            }
            2 => {
                ctx.set_i32_at(AppContext::SNIPER_RECOIL, -0xa)?;

                break 'sequence;
            }
            _ => ctx.set_i32_at(AppContext::SNIPER_RECOIL, 0)?,
        }

        if frame == 0xb {
            let aim = ctx.f32_at(AppContext::SNIPER_AIM_ANGLE)?;
            let goal = ctx.f32_at(AppContext::SNIPER_AIM_GOAL)?;

            ctx.set_i32_at(AppContext::SNIPER_ALIGNED, (aim == goal) as i32)?;

            let target = ctx.i32_at(AppContext::SNIPER_TARGET)?;

            if aim != goal
                || ctx.i32_at(AppContext::entity_field(1, target, Entity::OCCUPANT))? == 0
                || target == 0
                || ctx.i32_at(AppContext::entity_field(1, target, Entity::STATE))? > 2
            {
                ctx.set_i32_at(AppContext::SCRATCH_0, 0)?;
                ctx.set_block_at::<1>(AppContext::SNIPER_FIRING, [0])?;
                ctx.set_i32_at(AppContext::SNIPER_FIRE_FRAME, 0)?;
                ctx.set_i32_at(AppContext::SNIPER_CHARGE, 0x12c)?;

                break 'sequence;
            }

            ctx.set_i32_at(AppContext::SCRATCH_0, 1)?;

            let mut free = 0usize;

            while free != 50 && ctx.u8_at(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(free))? != 0 {
                free += 1;
            }

            if free == 50 {
                ctx.set_i32_at(AppContext::CAMERA_KICK, 0x14)?;

                break 'sequence;
            }

            ctx.set_block_at::<1>(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(free), [1])?;
            ctx.set_i32_at(AppContext::PENDING_STRIKE_SPEED.wrapping_add(free.wrapping_mul(4)), 0x5dc)?;

            let origin_x = ctx.i32_at(AppContext::SCRATCH_1)?;

            ctx.set_i32_at(AppContext::PENDING_STRIKE_TRIGGER_X.wrapping_add(free.wrapping_mul(4)), origin_x)?;

            let origin_y = ctx.i32_at(AppContext::SCRATCH_2)?;

            ctx.set_i32_at(AppContext::PENDING_STRIKE_Y.wrapping_add(free.wrapping_mul(4)), origin_y)?;
            ctx.set_i32_at(AppContext::PENDING_STRIKE_ANGLE.wrapping_add(free.wrapping_mul(4)), operation::cvttss2si(goal))?;
            ctx.set_i32_at(AppContext::PENDING_STRIKE_TARGET.wrapping_add(free.wrapping_mul(4)), target)?;
            ctx.set_i32_at(AppContext::SNIPER_CHARGE, 0)?;

            let seeded = [0i32, 0, 0, 0x1e, 0, 0, 0, 0, -0x1e, 0, 0, 0, 0, 0, 0x1e, 0, 0, 0, 0, -0x1e];
            let mut cell = 0usize;

            while cell != 20 {
                ctx.set_i32_at(AppContext::SNIPER_CASINGS.wrapping_add(cell.wrapping_mul(4)), *seeded.get(cell).ok_or(Fault::IndexOutOfRange { site: "sniper_update", index: cell as i64, limit: 20 })?)?;
                cell += 1;
            }

            ctx.set_block_at::<1>(AppContext::SNIPER_CASINGS_LIVE, [1])?;
            frame = ctx.i32_at(AppContext::SNIPER_FIRE_FRAME)?;
        }

        if frame >= 0xc && ctx.u8_at(AppContext::SNIPER_CASINGS_LIVE)? != 0 {
            let mut casing = 0usize;

            while casing != 4 {
                let record = AppContext::SNIPER_CASINGS.wrapping_add(casing.wrapping_mul(0x14));
                let drift_x = ctx.i32_at(record.wrapping_add(0xc))?;
                let drift_y = ctx.i32_at(record.wrapping_add(0x10))?;
                let x = ctx.i32_at(record)?.wrapping_add(drift_x);
                let y = ctx.i32_at(record.wrapping_add(4))?.wrapping_add(drift_y);

                ctx.set_i32_at(record, x)?;
                ctx.set_i32_at(record.wrapping_add(4), y)?;
                ctx.set_i32_at(record.wrapping_add(0xc), operation::cvttsd2si(drift_x as f64 * 0.5))?;
                ctx.set_i32_at(record.wrapping_add(0x10), operation::cvttsd2si(drift_y as f64 * 0.5))?;

                let age = ctx.i32_at(record.wrapping_add(8))?;

                ctx.set_i32_at(record.wrapping_add(8), age.wrapping_add(1))?;

                if age >= 9 {
                    ctx.set_block_at::<1>(AppContext::SNIPER_CASINGS_LIVE, [0])?;
                }

                casing += 1;
            }
        }

        match frame {
            0xb => ctx.set_i32_at(AppContext::CAMERA_KICK, 0x14)?,
            0xc => ctx.set_i32_at(AppContext::CAMERA_KICK, -0xa)?,
            0xd => ctx.set_i32_at(AppContext::CAMERA_KICK, 5)?,
            0xe => ctx.set_i32_at(AppContext::CAMERA_KICK, -2)?,
            0xf => ctx.set_i32_at(AppContext::CAMERA_KICK, 1)?,
            0x10 => ctx.set_i32_at(AppContext::CAMERA_KICK, 0)?,
            _ => {
                if frame >= 0x1c {
                    ctx.set_block_at::<1>(AppContext::SNIPER_FIRING, [0])?;
                }
            }
        }
    }

    let mut strike = 0usize;

    while strike != 50 {
        if ctx.u8_at(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike))? != 0 {
            let cell = strike.wrapping_mul(4);
            let speed = ctx.i32_at(AppContext::PENDING_STRIKE_SPEED.wrapping_add(cell))? as f32;
            let across = cos_deg(ctx.i32_at(AppContext::PENDING_STRIKE_ANGLE.wrapping_add(cell))? as f32);
            let x = operation::cvttss2si(ctx.i32_at(AppContext::PENDING_STRIKE_TRIGGER_X.wrapping_add(cell))? as f32 - across * speed);

            ctx.set_i32_at(AppContext::PENDING_STRIKE_TRIGGER_X.wrapping_add(cell), x)?;

            let speed = ctx.i32_at(AppContext::PENDING_STRIKE_SPEED.wrapping_add(cell))? as f32;
            let down = sin_deg(ctx.i32_at(AppContext::PENDING_STRIKE_ANGLE.wrapping_add(cell))? as f32);
            let y = operation::cvttss2si(ctx.i32_at(AppContext::PENDING_STRIKE_Y.wrapping_add(cell))? as f32 - down * speed);

            ctx.set_i32_at(AppContext::PENDING_STRIKE_Y.wrapping_add(cell), y)?;

            if operation::div_10(y) >= get_design_height2(ctx) {
                ctx.set_block_at::<1>(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike), [0])?;
            }

            if ctx.i32_at(AppContext::PENDING_STRIKE_TRIGGER_X.wrapping_add(cell))? <= -0x3e8 {
                ctx.set_block_at::<1>(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike), [0])?;
            }
        }

        strike += 1;
    }

    Ok(())
}
