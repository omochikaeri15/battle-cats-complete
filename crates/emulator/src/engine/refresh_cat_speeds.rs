use crate::Fault;

use super::{
    AppContext, get_button_unit_form, get_entity_button, get_slot_unit_id, get_speed, set_speed,
    slot_occupied, stat_speed,
};

pub fn refresh_cat_speeds(ctx: &mut AppContext) -> Result<(), Fault> {
    for slot in 1..0x33 {
        if slot_occupied(ctx, 0, slot)? == 0 {
            continue;
        }

        if get_speed(ctx, 0, slot)? == 0 {
            continue;
        }

        let unit_id = get_slot_unit_id(ctx, 0, slot)?;
        let button = get_entity_button(ctx, 0, slot)?;
        let form = get_button_unit_form(ctx, 0, button)?;
        let speed = stat_speed(ctx, 0, unit_id, form)?;

        set_speed(ctx, 0, slot, speed)?;
    }

    Ok(())
}
