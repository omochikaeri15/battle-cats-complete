use crate::Fault;

use super::{get_occupant, AppContext};

pub fn get_entity_base_idx(ctx: &AppContext) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::CASTLE_ENEMY_ROW)? <= 0 {
        return Ok(-1);
    }

    let mut slot = 0i32;

    while slot != 0x33 {
        if get_occupant(ctx, 1, slot)? == ctx.i32_at(AppContext::CASTLE_ENEMY_ROW)? {
            return Ok(slot);
        }

        slot += 1;
    }

    Ok(-1)
}
