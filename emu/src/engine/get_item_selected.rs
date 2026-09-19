use crate::Fault;

use super::{AppContext, get_map_type, labyrinth_active};

pub fn get_item_selected(ctx: &mut AppContext, item: i32) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? == -6 {
        return Ok(false);
    }

    let flags = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
        AppContext::ITEMS_SELECTED
    } else if labyrinth_active(ctx)? {
        AppContext::ITEMS_SELECTED_LABYRINTH
    } else {
        AppContext::ITEMS_SELECTED_SCORE_MODE
    };

    Ok(ctx.u8_at(flags.wrapping_add(item as i64 as usize))? != 0)
}
