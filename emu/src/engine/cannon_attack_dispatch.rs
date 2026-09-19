use crate::{operation, Fault};

use super::{
    attack_dmg_dispatch, attack_proc_dispatch, call_rng, cannon_hp_mode, cannon_makes_wave, does_target, get_barrier_hp, get_base_soulstrike, get_cannon_burrowed_permille,
    get_cannon_damage, get_cannon_hit_stamp, get_cannon_metal_permille, get_cannon_nonmetal_permille, get_cannon_nonzombie_permille, get_cannon_shot_id, get_cannon_type,
    get_cannon_zombie_permille, get_dodge_chance, get_dodge_duration, get_dodge_timer, get_entity_state, get_global_map_id, get_hp, get_max_hp, get_special_rule_params,
    get_wave_block, get_wave_immune, has_base_curse_chance, has_base_freeze_chance, has_base_slow_chance, has_cannon_recoil, is_metal, is_touchable_thunk, is_zombie, max_i32,
    set_barrier_state, set_cannon_blast_hit, set_cannon_hit_stamp, set_castle_anim_frame, set_castle_anim_state, set_dodge_fx_frame, set_dodge_timer, set_zkill_hit,
    std_map_int_string_subscript, AppContext, CannonShot, Entity,
};

pub fn cannon_attack_dispatch(ctx: &mut AppContext, faction: i32, target: i32, shot_id: i32) -> Result<(), Fault> {
    const SITE: &str = "cannon_attack_dispatch";

    let other = 1i32.wrapping_sub(faction);
    let burrowed = if get_cannon_type(ctx, faction)? == 5 { get_entity_state(ctx, other, target)? == 0xc } else { false };
    let spark = if get_base_soulstrike(ctx, faction)? && get_entity_state(ctx, other, target)? == 0xe { 2 } else { 0 };

    ctx.set_i32_at(AppContext::entity_field(other, target, Entity::HIT_SPARK_TYPE), spark)?;

    if !(burrowed | is_touchable_thunk(ctx, other, target, 0)?) {
        return Ok(());
    }

    if get_cannon_hit_stamp(ctx, other, target)? >= shot_id {
        return Ok(());
    }

    if does_target(ctx, other, target, 0)? {
        if get_dodge_timer(ctx, other, target)? > 0 {
            return Ok(());
        }

        let roll = call_rng(ctx, 100);

        if roll < get_dodge_chance(ctx, other, target)? {
            let duration = get_dodge_duration(ctx, other, target)?;

            set_dodge_timer(ctx, other, target, duration)?;

            return set_dodge_fx_frame(ctx, other, target, 1);
        }
    }

    if cannon_makes_wave(ctx, faction)? && get_wave_block(ctx, other, target)? {
        set_castle_anim_state(ctx, 0, 0)?;
        set_castle_anim_frame(ctx, 0, 0)?;

        if get_cannon_type(ctx, faction)? == 0 || get_cannon_type(ctx, faction)? == 5 {
            let shots = AppContext::CANNON_SHOTS.wrapping_add((faction as i64 as usize).wrapping_mul(AppContext::CANNON_SHOTS_FACTION_STRIDE));
            let mut shot = 0usize;

            while shot != 15 {
                ctx.set_i32_at(shots.wrapping_add(shot.wrapping_mul(AppContext::CANNON_SHOT_STRIDE)).wrapping_add(CannonShot::TIMER), 0)?;
                shot += 1;
            }
        }

        ctx.set_i32_at(AppContext::entity_field(other, target, Entity::WAVE_BLOCK_FX_FRAME), 0)?;
        ctx.set_i32_at(AppContext::entity_field(other, target, Entity::WAVE_BLOCK_FX_ACTIVE), 1)?;

        return Ok(());
    }

    let stamp = get_cannon_shot_id(ctx, faction)?;

    set_cannon_hit_stamp(ctx, other, target, stamp)?;

    if cannon_makes_wave(ctx, faction)? && get_wave_immune(ctx, other, target)? {
        ctx.set_i32_at(AppContext::entity_field(other, target, Entity::WAVE_IMMUNE_FX_FRAME), 0)?;
        ctx.set_i32_at(AppContext::entity_field(other, target, Entity::WAVE_IMMUNE_FX_ACTIVE), 1)?;

        return Ok(());
    }

    'damage: {
        let damage;

        if get_cannon_type(ctx, faction)? == 4 {
            let hp = if cannon_hp_mode(ctx, faction)? == 0 {
                get_hp(ctx, other, target)?
            } else if cannon_hp_mode(ctx, faction)? == 1 {
                get_max_hp(ctx, other, target)?
            } else {
                0
            };
            let cannon_type = get_cannon_type(ctx, faction)?;

            ctx.cannon_type_names.entry(cannon_type).or_default();

            if is_metal(ctx, other, target)? {
                damage = operation::div_1000(get_cannon_metal_permille(ctx, faction)?.wrapping_mul(hp));
                max_i32(damage, 1);
                get_cannon_metal_permille(ctx, faction)?;
            } else {
                damage = operation::div_1000(get_cannon_nonmetal_permille(ctx, faction)?.wrapping_mul(hp));
                max_i32(damage, 1);
                get_cannon_nonmetal_permille(ctx, faction)?;
            }
        } else if get_cannon_type(ctx, faction)? == 5 {
            let hp = if cannon_hp_mode(ctx, faction)? == 0 {
                get_hp(ctx, other, target)?
            } else if cannon_hp_mode(ctx, faction)? == 1 {
                get_max_hp(ctx, other, target)?
            } else {
                0
            };
            let cannon_type = get_cannon_type(ctx, faction)?;

            ctx.cannon_type_names.entry(cannon_type).or_default();

            if is_zombie(ctx, other, target)? {
                if get_entity_state(ctx, other, target)? == 0xb || get_entity_state(ctx, other, target)? == 0xc || get_entity_state(ctx, other, target)? == 0xd {
                    damage = operation::div_1000(get_cannon_burrowed_permille(ctx, faction)?.wrapping_mul(hp));
                } else {
                    damage = operation::div_1000(get_cannon_zombie_permille(ctx, faction)?.wrapping_mul(hp));
                }

                max_i32(damage, 1);
                set_zkill_hit(ctx, other, target, 1)?;
            } else {
                damage = operation::div_1000(get_cannon_nonzombie_permille(ctx, faction)?.wrapping_mul(hp));
                max_i32(damage, 1);
            }
        } else {
            if get_cannon_type(ctx, faction)? != 0 {
                let cannon_type = get_cannon_type(ctx, faction)?;

                std_map_int_string_subscript(&mut ctx.cannon_type_names, &cannon_type);
            }

            if get_cannon_damage(ctx, faction)? > 0 && is_metal(ctx, other, target)? {
                attack_dmg_dispatch(ctx, other, target, 1)?;

                break 'damage;
            }

            let mut plain = get_cannon_damage(ctx, faction)?;

            if get_cannon_type(ctx, faction)? == 0 {
                let map_id = get_global_map_id(ctx, 0)?;

                if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 9)? {
                    let percent = *params.first().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

                    plain = operation::div_100(plain.wrapping_mul(percent));
                }
            }

            attack_dmg_dispatch(ctx, other, target, plain)?;

            break 'damage;
        }

        attack_dmg_dispatch(ctx, other, target, if damage >= 2 { damage } else { 1 })?;
    }

    if has_cannon_recoil(ctx, faction)? {
        set_cannon_blast_hit(ctx, other, target, 1)?;
    }

    if get_barrier_hp(ctx, other, target)? > 0 && get_cannon_type(ctx, faction)? == 6 {
        set_barrier_state(ctx, other, target, 2)?;
    }

    let p_knockback = (get_cannon_type(ctx, faction)? == 6) as i32;
    let p_freeze = has_base_freeze_chance(ctx, faction)? as i32;
    let p_slow = has_base_slow_chance(ctx, faction)? as i32;
    let p_curse = has_base_curse_chance(ctx, faction)? as u8;

    attack_proc_dispatch(ctx, faction, 0, target, 0x19, p_knockback, p_freeze, p_slow, 0, 0, p_curse, 0)
}
