use crate::{Fault, ops};

use super::{AppContext, obf_value_read, xor_row_get};

const DISCOUNT: [f32; 2] = [1.0, 0.3];

pub fn get_miracle_price(ctx: &AppContext, miracle: i32) -> Result<i32, Fault> {
    let cell = ctx.block_at::<8>(
        AppContext::MIRACLE_PRICES.wrapping_add((miracle as i64 as usize).wrapping_mul(8)),
    )?;
    let price = obf_value_read(&cell) as i32 as f32;
    let chapter = xor_row_get(ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?, 7).ok_or(
        Fault::index_out_of_range(7, 10),
    )? as i32;

    Ok(ops::cvttsd2si(
        (price * DISCOUNT[(chapter >= 0x30) as usize]) as f64 + 0.5,
    ))
}
