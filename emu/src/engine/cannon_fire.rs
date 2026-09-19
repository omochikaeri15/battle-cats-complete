use crate::{operation, Fault};

use super::{
    add_cannon_shot_id, get_button_unit_form, get_button_unit_id, get_button_unit_row, get_cannon_countdown, get_cannon_recharge, get_cannon_strike_width, get_cannon_type,
    get_cannon_wall_hp_pct, get_cannon_wall_lifetime, get_cannon_wall_offset, get_castle_anim_state, get_max_hp, get_pos_x, get_tech_level, is_cannon_target, play_sound,
    set_cannon_countdown, set_cannon_strike_x, set_castle_anim_frame, set_castle_anim_state, set_death_timer, set_hp, set_max_hp, set_pos_x, slot_occupied, sound_manager,
    spawn_entity, AppContext, CatStats, CAT_STATS, CAT_STATS_FORM_STRIDE, CAT_STATS_UNIT_STRIDE,
};

pub fn cannon_fire(ctx: &mut AppContext, manual: u8) -> Result<(), Fault> {
    'refused: {
        if get_cannon_countdown(ctx, 0)? != 0 || get_castle_anim_state(ctx, 0)? != 0 {
            break 'refused;
        }

        ctx.set_block_at::<1>(AppContext::faction_flags(0).wrapping_add(AppContext::WALLET_CANNON_FIRED), [1])?;

        'fired: {
            let sound_id;

            if get_cannon_type(ctx, 0)? == 0 {
                set_castle_anim_state(ctx, 0, 1)?;
                set_castle_anim_frame(ctx, 0, 0)?;
                sound_id = 0x19;
            } else if get_cannon_type(ctx, 0)? == 1 {
                set_castle_anim_state(ctx, 0, 3)?;
                set_castle_anim_frame(ctx, 0, 0)?;
                sound_id = 0x3c;
            } else if get_cannon_type(ctx, 0)? == 7 {
                set_castle_anim_state(ctx, 0, 0xc)?;
                set_castle_anim_frame(ctx, 0, 0)?;
                sound_id = 0x7c;
            } else if get_cannon_type(ctx, 0)? == 2 {
                let row = get_button_unit_row(ctx, 0, 0xa)?;
                let unit_id = get_button_unit_id(ctx, 0, 0xa)?;
                let level = get_tech_level(ctx, ((unit_id as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize)?;
                let z_min_row = get_button_unit_row(ctx, 0, 0xa)? as i64;
                let z_min_form = get_button_unit_form(ctx, 0, 0xa)? as i64;
                let z_min = ctx.i32_at((z_min_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_min_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MINIMUM_Z_LAYER as i64) as usize)?;
                let z_max_row = get_button_unit_row(ctx, 0, 0xa)? as i64;
                let z_max_form = get_button_unit_form(ctx, 0, 0xa)? as i64;
                let z_max = ctx.i32_at((z_max_row * CAT_STATS_UNIT_STRIDE as i64 + CAT_STATS as i64 + z_max_form * CAT_STATS_FORM_STRIDE as i64 + CatStats::MAXIMUM_Z_LAYER as i64) as usize)?;
                let form = get_button_unit_form(ctx, 0, 0xa)?;
                let wall = spawn_entity(ctx, 0, row, level, z_min, z_max, form, 0)?;

                if wall < 0 {
                    break 'refused;
                }

                let mut front = 0xc80i32;
                let mut slot = 0i32;

                while slot != 51 {
                    if slot_occupied(ctx, 1, slot)? == 2 && get_pos_x(ctx, 1, slot)? > front {
                        front = get_pos_x(ctx, 1, slot)?;
                    }

                    slot += 1;
                }

                let x = front.wrapping_add(get_cannon_wall_offset(ctx, 0)?);

                set_pos_x(ctx, 0, wall, x)?;

                let full = get_max_hp(ctx, 0, wall)?;
                let scaled = operation::div_100(get_cannon_wall_hp_pct(ctx, 0)?.wrapping_mul(full));

                set_max_hp(ctx, 0, wall, scaled)?;

                let hp = get_max_hp(ctx, 0, wall)?;

                set_hp(ctx, 0, wall, hp)?;

                let lifetime = get_cannon_wall_lifetime(ctx, 0)?;

                set_death_timer(ctx, 0, wall, lifetime)?;
                set_castle_anim_state(ctx, 0, 6)?;
                set_castle_anim_frame(ctx, 0, 0)?;
                sound_id = 0x3d;
            } else if get_cannon_type(ctx, 0)? == 3 {
                let mut front = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0xc80);
                let mut slot = 0i32;

                while slot != 51 {
                    if slot_occupied(ctx, 0, slot)? == 2 && get_pos_x(ctx, 0, slot)? < front && is_cannon_target(ctx, 0, slot)? {
                        front = get_pos_x(ctx, 0, slot)?;
                    }

                    slot += 1;
                }

                set_castle_anim_state(ctx, 0, 4)?;
                set_castle_anim_frame(ctx, 0, 0)?;

                let strike_x = front.wrapping_sub(operation::div_2(get_cannon_strike_width(ctx, 0)?));

                set_cannon_strike_x(ctx, 0, strike_x)?;
                sound_id = 0x25;
            } else if get_cannon_type(ctx, 0)? == 4 {
                let mut front = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_add(-0xc80);
                let mut slot = 0i32;

                while slot != 51 {
                    if slot_occupied(ctx, 0, slot)? == 2 && get_pos_x(ctx, 0, slot)? < front && is_cannon_target(ctx, 0, slot)? {
                        front = get_pos_x(ctx, 0, slot)?;
                    }

                    slot += 1;
                }

                set_castle_anim_state(ctx, 0, 7)?;
                set_castle_anim_frame(ctx, 0, 0)?;

                let strike_x = front.wrapping_sub(operation::div_2(get_cannon_strike_width(ctx, 0)?));

                set_cannon_strike_x(ctx, 0, strike_x)?;
                sound_id = 0x41;
            } else if get_cannon_type(ctx, 0)? == 5 {
                set_castle_anim_state(ctx, 0, 9)?;
                set_castle_anim_frame(ctx, 0, 0)?;
                sound_id = 0x54;
            } else {
                if get_cannon_type(ctx, 0)? == 6 {
                    let mut front = 0xc80i32;
                    let mut slot = 0i32;

                    while slot != 51 {
                        if slot_occupied(ctx, 1, slot)? == 2 && get_pos_x(ctx, 1, slot)? > front {
                            front = get_pos_x(ctx, 1, slot)?;
                        }

                        slot += 1;
                    }

                    set_castle_anim_state(ctx, 0, 0xb)?;
                    set_castle_anim_frame(ctx, 0, 0)?;
                    set_cannon_strike_x(ctx, 0, front)?;
                }

                break 'fired;
            }

            play_sound(sound_manager(ctx)?, sound_id, None);
        }

        let recharge = get_cannon_recharge(ctx, 0)?;

        set_cannon_countdown(ctx, 0, recharge)?;
        add_cannon_shot_id(ctx, 0, 1)?;
        play_sound(sound_manager(ctx)?, 0x13, None);
        ctx.set_i32_at(AppContext::UI_TAP_LOCKOUT, 0)?;
        ctx.set_block_at::<1>(AppContext::INPUT_BLOCKED, [0])?;
        ctx.set_i32_at(AppContext::CPU_PENDING_ACTION, 0)?;
        ctx.set_block_at::<1>(AppContext::MISSION_CANNON_FIRED, [1])?;

        return Ok(());
    }

    if manual != 0 {
        play_sound(sound_manager(ctx)?, 0xf, None);
    }

    Ok(())
}
