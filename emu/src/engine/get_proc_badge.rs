use crate::Fault;

use super::{AppContext, PROC_BADGE_FIELDS};

pub fn get_proc_badge(ctx: &AppContext, faction: i32, slot: i32, index: i32) -> Result<i32, Fault> {
    let field = *PROC_BADGE_FIELDS
        .get(index as usize)
        .ok_or(Fault::IndexOutOfRange { site: "get_proc_badge", index: index as i64, limit: 5 })?;

    ctx.i32_at(AppContext::entity_field(faction, slot, 0x838f8).wrapping_add((field as usize).wrapping_mul(4)))
}
