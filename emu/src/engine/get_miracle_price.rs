use crate::{operation, Fault};

use super::{obf_value_read, xor_row_get, AppContext};

const SITE: &str = "get_miracle_price";

const DISCOUNT: [f32; 2] = [1.0, 0.3];

pub fn get_miracle_price(ctx: &AppContext, miracle: i32) -> Result<i32, Fault> {
    let cell = ctx.block_at::<8>(AppContext::MIRACLE_PRICES.wrapping_add((miracle as i64 as usize).wrapping_mul(8)))?;
    let price = obf_value_read(&cell) as i32 as f32;
    let chapter = xor_row_get(ctx.bytes_from(AppContext::CHAPTER_PROGRESS)?, 7).ok_or(Fault::IndexOutOfRange { site: SITE, index: 7, limit: 10 })? as i32;

    Ok(operation::cvttsd2si((price * DISCOUNT[(chapter >= 0x30) as usize]) as f64 + 0.5))
}
