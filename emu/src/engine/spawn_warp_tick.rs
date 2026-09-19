use crate::Fault;

use super::{
    add_entity_frame, add_warp_timer, advance_animation_frame, can_push_back, get_entity_frame, get_entity_state, get_pos_x,
    get_warp_distance, get_warp_timer, keep_in_bound, play_sound_in_battle, read_flag, set_entity_frame, set_entity_state,
    set_pos_x, AppContext,
};

pub fn spawn_warp_tick(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    if get_entity_state(ctx, faction, slot)? == 0xa {
        advance_animation_frame(ctx, faction, slot)?;

        if get_entity_frame(ctx, faction, slot)? == 0 {
            return set_entity_state(ctx, faction, slot, 0);
        }

        return Ok(());
    }

    if get_entity_state(ctx, faction, slot)? == 0x12 {
        add_entity_frame(ctx, faction, slot, 1)?;

        if get_entity_frame(ctx, faction, slot)? >= 0x1e {
            return set_entity_state(ctx, faction, slot, 0x13);
        }

        return Ok(());
    }

    if get_entity_state(ctx, faction, slot)? == 0x13 {
        if get_warp_timer(ctx, faction, slot)? <= 0 {
            set_entity_state(ctx, faction, slot, 0x14)?;
            play_sound_in_battle(ctx, 0x4a)?;

            if can_push_back(ctx, faction, slot)? {
                let cat_side = read_flag(ctx, AppContext::faction_flags(faction))?;
                let x = get_pos_x(ctx, faction, slot)?;
                let distance = get_warp_distance(ctx, faction, slot)?;
                let step = if cat_side & 1 != 0 { distance } else { distance.wrapping_neg() };

                set_pos_x(ctx, faction, slot, step.wrapping_add(x))?;
                keep_in_bound(ctx, faction, slot)?;
            }
        } else {
            add_warp_timer(ctx, faction, slot, -1)?;
        }

        return set_entity_frame(ctx, faction, slot, 0);
    }

    if get_entity_state(ctx, faction, slot)? != 0x14 {
        return Ok(());
    }

    add_entity_frame(ctx, faction, slot, 1)?;

    if get_entity_frame(ctx, faction, slot)? >= 0x14 {
        set_entity_state(ctx, faction, slot, 0)?;

        return set_entity_frame(ctx, faction, slot, 0);
    }

    Ok(())
}
