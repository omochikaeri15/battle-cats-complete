use crate::Fault;

use super::{read_flag, AppContext};

pub fn is_relic(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    let faction_index = faction as usize;

    if read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, 0x83b40))? != 0)
}
