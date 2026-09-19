use crate::Fault;

use super::{treasure_group_at, AppContext, STAGE_DISPLAY_ORDER};

const SITE: &str = "treasure_group_of_stage";

pub fn treasure_group_of_stage(ctx: &AppContext, chapter: i32, stage: i32) -> Result<i32, Fault> {
    if chapter == 3 {
        return Ok(0);
    }

    let groups = ctx.treasure_store.get(chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?.len() as i32;

    if groups <= 0 {
        return Ok(0);
    }

    let mut group = 0;

    loop {
        let mut index = 0i64;

        loop {
            let count = treasure_group_at(&ctx.treasure_store, chapter, group)?.count as i64;

            if index >= count {
                break;
            }

            let castles = treasure_group_at(&ctx.treasure_store, chapter, group)?.castles;
            let castle = *castles.get(index as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index, limit: castles.len() as i64 })?;
            let wanted = *STAGE_DISPLAY_ORDER.get(stage as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: stage as i64, limit: 0x33 })?;

            index += 1;

            if castle == wanted {
                return Ok(group);
            }
        }

        group += 1;

        let groups = ctx.treasure_store.get(chapter as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: chapter as i64, limit: 10 })?.len() as i32;

        if group >= groups {
            return Ok(0);
        }
    }
}
