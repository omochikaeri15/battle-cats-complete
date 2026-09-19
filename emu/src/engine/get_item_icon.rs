use crate::Fault;

use super::{AppContext, ItemDefinition};

pub fn get_item_icon(ctx: &AppContext, item: i32) -> Result<i32, Fault> {
    if item as u32 > 0x112 {
        return Ok(-1);
    }

    let icon = ctx.i32_at(
        AppContext::ITEM_DEFINITIONS
            + (item as i64 as usize) * AppContext::ITEM_DEFINITION_STRIDE
            + ItemDefinition::ICON,
    )?;

    Ok(if icon == -1 { 0x78 } else { icon })
}
