use crate::Fault;

use super::{AppContext, obf_value_read, obf_value_sub, resource_log};

pub fn spend_cat_food(ctx: &mut AppContext, amount: i32) -> Result<bool, Fault> {
    if amount < 0 {
        return Ok(false);
    }

    let mut cell = ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?;

    if (obf_value_read(&cell) as i32) < amount {
        return Ok(false);
    }

    obf_value_sub(&mut cell, amount);
    ctx.set_block_at(AppContext::ITEM_16_COUNT, cell)?;
    resource_log(ctx, b"use", b"catfood", amount)?;

    Ok(true)
}
