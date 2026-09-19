use crate::Fault;

use super::AppContext;

pub fn get_listed_item_count(ctx: &AppContext, index: i32) -> Result<i32, Fault> {
    let mut count = 0i32;

    if index >= 0 && (ctx.listed_item_counts.len() as u64 as i32) > index {
        count =
            ctx.listed_item_counts
                .get(index as u32 as usize)
                .ok_or(Fault::IndexOutOfRange {
                    site: "get_listed_item_count",
                    index: index as i64,
                    limit: ctx.listed_item_counts.len() as i64,
                })?[1];
    }

    Ok(count)
}
