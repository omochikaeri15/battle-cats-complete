use crate::Fault;

use super::{get_entity_button, get_slot_unit_id, get_unit_rarity, slot_occupied, AppContext};

pub fn recount_deploy_rarities(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.zero(AppContext::DEPLOY_LIMIT_RARITY_COUNTS, 0x18)?;

    let mut slot = 1i32;

    while slot != 51 {
        if slot_occupied(ctx, 0, slot)? == 2 && get_entity_button(ctx, 0, slot)? <= 9 {
            let rarity = get_unit_rarity(ctx, get_slot_unit_id(ctx, 0, slot)?)? as i64;
            let cell = (rarity * 4 + AppContext::DEPLOY_LIMIT_RARITY_COUNTS as i64) as usize;
            let count = ctx.i32_at(cell)?;

            ctx.set_i32_at(cell, count.wrapping_add(1))?;
        }

        slot += 1;
    }

    Ok(())
}
