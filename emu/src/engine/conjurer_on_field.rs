use crate::Fault;

use super::{get_button_unit_row, get_entity_state, get_slot_unit_id, AppContext};

pub fn conjurer_on_field(ctx: &AppContext, faction: i32, slot: i32, check_lockout: i32) -> Result<bool, Fault> {
    let wallet = AppContext::faction_flags(faction);

    if check_lockout != 0 && ctx.i32_at(wallet.wrapping_add(((slot as i64) * 4) as usize).wrapping_add(AppContext::WALLET_CONJURE_LOCKOUT))? > 0 {
        return Ok(false);
    }

    if ctx.u8_at(wallet.wrapping_add(slot as i64 as usize).wrapping_add(AppContext::WALLET_SPIRIT_USED))? != 0 {
        return Ok(false);
    }

    let unit_id = get_button_unit_row(ctx, faction, slot)?.wrapping_add(-2);
    let mut scan = 0i32;

    while scan != 51 {
        if get_slot_unit_id(ctx, faction, scan)? == unit_id && get_entity_state(ctx, faction, scan)? != 4 {
            return Ok(true);
        }

        scan += 1;
    }

    Ok(false)
}
