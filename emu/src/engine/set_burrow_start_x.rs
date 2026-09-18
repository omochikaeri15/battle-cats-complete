use crate::Fault;

use super::{read_flag, AppContext};

pub fn set_burrow_start_x(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    let faction_index = faction as usize;

    if read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83a80), value)
}
