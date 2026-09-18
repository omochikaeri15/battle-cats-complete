use crate::{operation, Fault};

use super::AppContext;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TreasureGroup {
    pub effect: i32,
    pub percent: i32,
    pub chapter_only: u8,
}

pub type TreasureStore = [Vec<TreasureGroup>; 10];

pub fn get_treasure_value(ctx: &AppContext, store: &TreasureStore, effect: i32) -> Result<i32, Fault> {
    let mut total = 0i32;
    let mut progress_row = 0x83708usize;

    for (chapter, groups) in store.iter().enumerate() {
        for (group_index, group) in groups.iter().enumerate() {
            if group.effect != effect {
                continue;
            }

            if group.chapter_only != 0 && chapter as u64 != ctx.i32_at(0x327efc)? as u32 as u64 {
                continue;
            }

            let percent = group.percent;
            let progress = ctx.i32_at(progress_row.wrapping_add(group_index * 4))?;

            if percent == 0 {
                total = total.wrapping_add(progress);
            } else {
                total = operation::div_100(progress.wrapping_mul(percent)).wrapping_add(total);
            }
        }

        progress_row = progress_row.wrapping_add(0x2c);
    }

    Ok(total)
}
