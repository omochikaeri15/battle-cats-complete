use crate::Fault;

use super::{get_map_type, labyrinth_active, AppContext};

pub fn powerup_available(ctx: &mut AppContext, powerup: i32) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? == -6 {
        return Ok(false);
    }

    let mut selected = false;

    if get_map_type(ctx, 0)? != -6 {
        let flags = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
            AppContext::ITEMS_SELECTED
        } else if labyrinth_active(ctx)? {
            AppContext::ITEMS_SELECTED_LABYRINTH
        } else {
            AppContext::ITEMS_SELECTED_SCORE_MODE
        };

        selected = ctx.u8_at(flags.wrapping_add(powerup as i64 as usize))? != 0;
    }

    Ok(powerup == 0 || selected)
}
