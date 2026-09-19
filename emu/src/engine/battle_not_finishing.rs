use crate::Fault;

use super::{AppContext, scored_map_pays_money};

pub fn battle_not_finishing(ctx: &mut AppContext) -> Result<bool, Fault> {
    if !scored_map_pays_money(ctx)? {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::BATTLE_STATUS)? != 4)
}
