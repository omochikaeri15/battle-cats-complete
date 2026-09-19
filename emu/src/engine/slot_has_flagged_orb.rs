use crate::Fault;

use super::{
    AppContext, get_button_unit_id, get_equipped_orb, get_orb_def, get_orb_slot_count,
    orb_ability_flag,
};

pub fn slot_has_flagged_orb(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    let unit_id = get_button_unit_id(ctx, faction, slot)?;
    let mut orb_slot = 0;

    if get_orb_slot_count(&ctx.orb_store, unit_id)? <= 0 {
        return Ok(false);
    }

    loop {
        let orb = get_equipped_orb(ctx, unit_id, orb_slot)?;

        if orb != -1 {
            let abil = get_orb_def(&ctx.orb_store, orb)?.abil;

            if orb_ability_flag(&mut ctx.orb_store, abil) {
                return Ok(true);
            }
        }

        orb_slot += 1;

        if orb_slot >= get_orb_slot_count(&ctx.orb_store, unit_id)? {
            return Ok(false);
        }
    }
}
