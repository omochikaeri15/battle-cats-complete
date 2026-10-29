use crate::Fault;

use super::{AppContext, get_map_type, labyrinth_active};

pub fn set_item_selected(ctx: &mut AppContext, item: i32, value: u8) -> Result<(), Fault> {
    if get_map_type(ctx, 0)? != -6 {
        let flags = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
            AppContext::ITEMS_SELECTED
        } else if labyrinth_active(ctx)? {
            AppContext::ITEMS_SELECTED_LABYRINTH
        } else {
            AppContext::ITEMS_SELECTED_SCORE_MODE
        };

        ctx.set_block_at::<1>(flags.wrapping_add(item as i64 as usize), [value])?;
    }

    let slot = match item {
        0 if value == 0 => AppContext::POWERUP_AVAILABLE,
        3 if value == 0 => AppContext::POWERUP_CLEARED,
        _ => return Ok(()),
    };
    let mode = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        labyrinth_active(ctx)? as usize + 1
    } else {
        0
    };

    ctx.set_block_at::<1>(slot + mode, [0])
}
