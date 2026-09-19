use crate::{operation, Fault};

use super::{treasure_group_at, AppContext};

const SITE: &str = "calculate_treasure_percentages";

pub fn calculate_treasure_percentages(ctx: &mut AppContext) -> Result<(), Fault> {
    for chapter in 0..10i32 {
        for group in 0..0xbi32 {
            let cell = AppContext::TREASURE_PROGRESS + chapter as usize * 0x2c + group as usize * 4;

            ctx.set_i32_at(cell, 0)?;

            let mut slot = 0usize;
            let mut sum = 0i32;

            let summed = 'sum: loop {
                if chapter != 3 {
                    let castles = treasure_group_at(&ctx.treasure_store, chapter, group)?.castles;
                    let castle = *castles.get(slot).ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: castles.len() as i64 })?;

                    if castle == -1 {
                        break 'sum true;
                    }

                    let row = ctx.bytes_from(AppContext::TREASURE_LEVELS + chapter as usize * AppContext::TREASURE_LEVELS_STRIDE)?;
                    let level = operation::xor_row_decode(row, 0x31, castle as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: castle as i64, limit: 0x31 })? as i32;

                    if level <= 0 {
                        break 'sum false;
                    }

                    let row = ctx.bytes_from(AppContext::TREASURE_LEVELS + chapter as usize * AppContext::TREASURE_LEVELS_STRIDE)?;

                    sum = sum.wrapping_add(operation::xor_row_decode(row, 0x31, castle as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: castle as i64, limit: 0x31 })? as i32);
                }

                slot += 1;

                if slot == 8 {
                    break 'sum true;
                }
            };

            if !summed || chapter == 3 {
                continue;
            }

            if sum != 0 {
                let count = treasure_group_at(&ctx.treasure_store, chapter, group)?.count;
                let divisor = count.wrapping_mul(3);
                let percent = operation::idiv(sum.wrapping_mul(100), divisor).ok_or(Fault::divide(SITE, divisor as i64))?;

                ctx.set_i32_at(cell, percent)?;
            }
        }
    }

    Ok(())
}
