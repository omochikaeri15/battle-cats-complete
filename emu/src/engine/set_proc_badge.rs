use crate::Fault;

use super::{AppContext, PROC_BADGE_FIELDS};

pub fn set_proc_badge(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    index: i32,
    value: i32,
) -> Result<(), Fault> {
    let field = *PROC_BADGE_FIELDS
        .get(index as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "set_proc_badge",
            index: index as i64,
            limit: 5,
        })?;

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)),
        value,
    )
}
