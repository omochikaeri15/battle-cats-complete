use crate::Fault;

use super::{AppContext, TreasureStore};

pub fn get_treasure_uncapped(
    ctx: &AppContext,
    store: &TreasureStore,
    effect: i32,
) -> Result<i32, Fault> {
    let mut total = 0i32;

    for (chapter, groups) in store.iter().enumerate() {
        for group in groups {
            if group.effect != effect {
                continue;
            }

            if group.chapter_only == 0
                || chapter as u64 == ctx.i32_at(AppContext::CHAPTER_MODE)? as u32 as u64
            {
                total = total.wrapping_add(group.percent);
            }
        }
    }

    Ok(total)
}
