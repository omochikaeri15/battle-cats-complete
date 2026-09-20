use crate::{Fault, ops};

use super::AppContext;

pub fn treasure_festival_active(ctx: &AppContext) -> Result<bool, Fault> {
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let group = (chapter <= 2 && ctx.i32_at(AppContext::FESTIVAL_EOC)? == 2)
        || ((chapter.wrapping_sub(4) as u32) <= 2 && ctx.i32_at(AppContext::FESTIVAL_ITF)? == 2);

    if group {
        if ctx.i32_at(AppContext::FIRST_STAGE_WON)? < 2 {
            return Ok(false);
        }
    } else if chapter < 7
        || ctx.i32_at(AppContext::FESTIVAL_COTC)? != 2
        || ctx.i32_at(AppContext::FIRST_STAGE_WON)? <= 1
    {
        return Ok(false);
    }

    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS)?);
    pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS_KEY)?);

    Ok(
        (ops::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32)
            > 0,
    )
}
