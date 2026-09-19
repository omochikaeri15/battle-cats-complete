use crate::{operation, Fault};

use super::AppContext;

const SITE: &str = "eoc_progress_total";

pub fn eoc_progress_total(ctx: &AppContext) -> Result<i32, Fault> {
    let mut total = 0i32;

    for chapter in 0..3usize {
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS + chapter * 4)?);
        pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS_KEY)?);

        let cleared = operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 1 })? as i32;

        if cleared == 0 {
            return Ok(total);
        }

        total = if cleared == 0x30 { (chapter as i32 + 1) * 100 } else { (chapter as i32 * 100).wrapping_add(cleared) };
    }

    Ok(total)
}
