use crate::Fault;

use super::{AppContext, treasure_group_at};

const SITE: &str = "treasure_group_cleared";

pub fn treasure_group_cleared(
    ctx: &mut AppContext,
    chapter: i32,
    group: i32,
) -> Result<bool, Fault> {
    if chapter == 3 {
        return Ok(false);
    }

    let mut index = 0i64;

    loop {
        let count = treasure_group_at(&ctx.treasure_store, chapter, group)?.count as i64;

        if index >= count {
            return Ok(true);
        }

        let castles = treasure_group_at(&ctx.treasure_store, chapter, group)?.castles;
        let castle = *castles.get(index as usize).ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index,
            limit: castles.len() as i64,
        })?;
        let stage = if castle > 0x2d { castle } else { 0x2d - castle };

        if !*ctx
            .outbreak_cleared
            .entry(chapter)
            .or_default()
            .entry(stage)
            .or_insert(false)
        {
            return Ok(false);
        }

        index += 1;
    }
}
