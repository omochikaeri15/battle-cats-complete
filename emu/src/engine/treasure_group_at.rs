use crate::Fault;

use super::{TreasureGroup, TreasureStore};

const SITE: &str = "treasure_group_at";

pub fn treasure_group_at(
    store: &TreasureStore,
    chapter: i32,
    index: i32,
) -> Result<TreasureGroup, Fault> {
    let groups = store
        .get(chapter as i64 as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: chapter as i64,
            limit: 10,
        })?;

    groups
        .get(index as i64 as usize)
        .cloned()
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: index as i64,
            limit: groups.len() as i64,
        })
}
