use crate::{Fault, operation};

use super::{
    AppContext, add_entity_frame, get_anim_len, get_entity_button, get_entity_frame,
    get_entity_state, get_spawn_anim_flag, get_spawn_anim_type, get_unit_anim,
    maanim_get_max_keyframe, set_entity_frame, std_map_int_maanim_subscript_2,
};

const SITE: &str = "advance_animation_frame";

pub fn advance_animation_frame(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    let button = get_entity_button(ctx, faction, slot)?;

    if get_entity_state(ctx, faction, slot)? == 0 {
        return add_entity_frame(ctx, faction, slot, 1);
    }

    if get_entity_state(ctx, faction, slot)? == 1 {
        let anim =
            get_unit_anim(ctx, faction, button, 1)?.ok_or(Fault::NullPointer { site: SITE })?;

        if maanim_get_max_keyframe(anim)? == 0 {
            return set_entity_frame(ctx, faction, slot, 0);
        }

        return add_entity_frame(ctx, faction, slot, 1);
    }

    if get_entity_state(ctx, faction, slot)? == 2 {
        let anim =
            get_unit_anim(ctx, faction, button, 2)?.ok_or(Fault::NullPointer { site: SITE })?;
        let length = get_anim_len(anim)?;

        if faction == 0 && length == -1 {
            let next = get_entity_frame(ctx, 0, slot)?.wrapping_add(1);
            let anim =
                get_unit_anim(ctx, 0, button, 2)?.ok_or(Fault::NullPointer { site: SITE })?;
            let last = maanim_get_max_keyframe(anim)?;
            let frame = operation::irem(next, last).ok_or(Fault::divide(SITE, last as i64))?;

            return set_entity_frame(ctx, 0, slot, frame);
        }

        let next = get_entity_frame(ctx, faction, slot)?.wrapping_add(1);
        let anim =
            get_unit_anim(ctx, faction, button, 2)?.ok_or(Fault::NullPointer { site: SITE })?;
        let length = get_anim_len(anim)?;
        let frame = operation::irem(next, length).ok_or(Fault::divide(SITE, length as i64))?;

        return set_entity_frame(ctx, faction, slot, frame);
    }

    if get_entity_state(ctx, faction, slot)? == 3 {
        let current = get_entity_frame(ctx, faction, slot)?;
        let cycles = operation::div_24(current.wrapping_add(1) as i64) as i32;
        let frame = current
            .wrapping_add(cycles.wrapping_shl(3).wrapping_mul(3).wrapping_neg())
            .wrapping_add(1);

        return set_entity_frame(ctx, faction, slot, frame);
    }

    if get_entity_state(ctx, faction, slot)? == 4 {
        return add_entity_frame(ctx, faction, slot, 1);
    }

    if get_entity_state(ctx, faction, slot)? == 0x15 {
        return add_entity_frame(ctx, faction, slot, 1);
    }

    if get_entity_state(ctx, faction, slot)? == 5 || get_entity_state(ctx, faction, slot)? == 6 {
        let current = get_entity_frame(ctx, faction, slot)?;
        let cycles = operation::div_12(current.wrapping_add(1) as i64) as i32;
        let frame = current
            .wrapping_add(cycles.wrapping_shl(2).wrapping_mul(3).wrapping_neg())
            .wrapping_add(1);

        return set_entity_frame(ctx, faction, slot, frame);
    }

    if get_entity_state(ctx, faction, slot)? == 7 {
        let current = get_entity_frame(ctx, faction, slot)?;
        let cycles = operation::div_48(current.wrapping_add(1) as i64) as i32;
        let frame = current
            .wrapping_add(cycles.wrapping_shl(4).wrapping_mul(3).wrapping_neg())
            .wrapping_add(1);

        return set_entity_frame(ctx, faction, slot, frame);
    }

    if get_entity_state(ctx, faction, slot)? == 0xa {
        let mut length = 1;

        if get_spawn_anim_flag(ctx, faction, slot)? {
            let anim =
                get_unit_anim(ctx, faction, button, 7)?.ok_or(Fault::NullPointer { site: SITE })?;
            let spawn_length = get_anim_len(anim)?;

            length = if spawn_length >= 2 { spawn_length } else { 1 };
        }

        if get_spawn_anim_type(ctx, faction, slot)? >= 0 {
            let key = get_spawn_anim_type(ctx, faction, slot)?;
            let effect_length =
                get_anim_len(std_map_int_maanim_subscript_2(&mut ctx.effect_anims, &key))?;

            if effect_length > length {
                length = effect_length;
            }
        }

        let next = get_entity_frame(ctx, faction, slot)?.wrapping_add(1);
        let frame = operation::irem(next, length).ok_or(Fault::divide(SITE, length as i64))?;

        set_entity_frame(ctx, faction, slot, frame)?;
    }

    Ok(())
}
