use crate::Fault;

use super::{read_flag, AppContext};

pub fn add_revive_timer(ctx: &mut AppContext, faction: i32, slot: i32, delta: i32) -> Result<(), Fault> {
    let faction_index = faction as usize;

    if read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        return Ok(());
    }

    let field = AppContext::entity_field(faction, slot, 0x83c2c);
    let current = ctx.i32_at(field)?;

    ctx.set_i32_at(field, current.wrapping_add(delta))
}
