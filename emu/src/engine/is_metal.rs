use crate::Fault;

use super::{read_flag, AppContext};

pub fn is_metal(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    let faction_index = faction as usize;
    let cat_side = read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0;
    let field = if cat_side { 0x83a20 } else { 0x839a4 };

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, field))? != 0)
}
