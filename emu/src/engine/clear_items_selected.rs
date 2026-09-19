use crate::Fault;

use super::{AppContext, labyrinth_active};

pub fn clear_items_selected(ctx: &mut AppContext) -> Result<(), Fault> {
    for item in 0..10usize {
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED + item, [0])?;
    }

    for item in 0..6usize {
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED_SCORE_MODE + item, [0])?;
    }

    for item in 0..6usize {
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED_LABYRINTH + item, [0])?;
    }

    let held = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
        AppContext::ITEM_HOLD_NORMAL
    } else if labyrinth_active(ctx)? {
        AppContext::ITEM_HOLD_LABYRINTH
    } else {
        AppContext::ITEM_HOLD_SCORE_MODE
    };

    if ctx.u8_at(held)? != 0 {
        return Ok(());
    }

    let mode = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        labyrinth_active(ctx)? as usize + 1
    } else {
        0
    };

    ctx.set_block_at::<1>(AppContext::POWERUP_AVAILABLE + mode, [0])?;

    let mode = if ctx.u8_at(AppContext::SCORE_MODE_FLAG)? != 0 {
        labyrinth_active(ctx)? as usize + 1
    } else {
        0
    };

    ctx.set_block_at::<1>(AppContext::POWERUP_CLEARED + mode, [0])
}
