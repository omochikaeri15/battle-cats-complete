use crate::Fault;

use super::AppContext;

pub fn find_item_by_kind(ctx: &AppContext, kind: i32, id: i32) -> Result<i32, Fault> {
    for index in 0..0x113usize {
        let row = AppContext::ITEM_DEFINITIONS + index * AppContext::ITEM_DEFINITION_STRIDE;

        if ctx.i32_at(row)? == kind && ctx.i32_at(row + 4)? == id {
            return Ok(index as i32);
        }
    }

    Ok(-1)
}
