use crate::{Fault, operation};

use super::{AppContext, TreasureStore};

pub fn get_treasure_capped(
    ctx: &AppContext,
    store: &TreasureStore,
    effect: i32,
    percent: i32,
) -> Result<i32, Fault> {
    let mut total = 0i32;
    let mut progress_row = AppContext::TREASURE_PROGRESS;

    for (chapter, groups) in store.iter().enumerate() {
        for (group_index, group) in groups.iter().enumerate() {
            if group.effect != effect {
                continue;
            }

            if group.chapter_only != 0
                && chapter as u64 != ctx.i32_at(AppContext::CHAPTER_MODE)? as u32 as u64
            {
                continue;
            }

            let progress = ctx.i32_at(progress_row.wrapping_add(group_index * 4))?;

            total = operation::div_100(progress.wrapping_mul(percent)).wrapping_add(total);
        }

        progress_row = progress_row.wrapping_add(0x2c);
    }

    Ok(total)
}
