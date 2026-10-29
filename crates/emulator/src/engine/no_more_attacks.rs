use crate::Fault;

use super::{
    AppContext, get_attack_end_mode, get_entity_state, get_speed, set_entity_frame,
    set_entity_state, set_speed,
};

pub fn no_more_attacks(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    if get_attack_end_mode(ctx, faction, slot)? == 0
        && (get_entity_state(ctx, faction, slot)? != 1 || get_speed(ctx, faction, slot)? != 0)
    {
        set_entity_state(ctx, faction, slot, 1)?;
        set_entity_frame(ctx, faction, slot, 0)?;

        return set_speed(ctx, faction, slot, 0);
    }

    if get_attack_end_mode(ctx, faction, slot)? == 2
        && get_entity_state(ctx, faction, slot)? != 4
        && get_entity_state(ctx, faction, slot)? != 0x15
    {
        set_entity_state(ctx, faction, slot, 4)?;

        return set_entity_frame(ctx, faction, slot, 0);
    }

    Ok(())
}
