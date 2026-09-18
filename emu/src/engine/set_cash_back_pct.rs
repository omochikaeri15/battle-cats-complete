use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn set_cash_back_pct(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::CASH_BACK_PCT), value)
}
