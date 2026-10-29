use crate::Fault;

use super::{TreasureGroup, TreasureStore};

pub fn treasure_group_at(
    store: &TreasureStore,
    chapter: i32,
    index: i32,
) -> Result<TreasureGroup, Fault> {
    let groups = store
        .get(chapter as i64 as usize)
        .ok_or(Fault::index_out_of_range(chapter as i64, 10))?;

    groups
        .get(index as i64 as usize)
        .cloned()
        .ok_or(Fault::index_out_of_range(index as i64, groups.len() as i64))
}
