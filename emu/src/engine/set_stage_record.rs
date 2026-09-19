use crate::Fault;

use super::{map_type_as_index, map_type_base_id, AppContext};

const SITE: &str = "set_stage_record";

pub fn set_stage_record(ctx: &mut AppContext, map_type: i32, map_idx: i32, stage: i32, star: i32, value: i32, use_cache: i32) -> Result<(), Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);
        let stars = ctx.stage_record_cache.entry(map_id).or_default().entry(stage).or_default();
        let record = stars.get_mut(star as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: star as i64, limit: 4 })?;

        *record = value as i16;

        return Ok(());
    }

    if map_type as u32 <= 4 {
        let cell = (map_type_as_index(map_type) as i64) * 0xbb80
            + (map_idx as i64) * 0x60
            + (stage as i64) * 8
            + (star as i64) * 2
            + AppContext::STAGE_RECORD_STORY as i64;

        return ctx.set_block_at::<2>(cell as usize, (value as i16).to_le_bytes());
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &mut ctx.stage_record_neg26,
                0x02 => &mut ctx.stage_record_neg24,
                0x03 => &mut ctx.stage_record_neg23,
                _ => &mut ctx.stage_record_neg22,
            };
            let limit = maps.len() as i64;
            let record = maps
                .get_mut(map_idx as i64 as usize)
                .and_then(|stars| stars.get_mut(star as i64 as usize))
                .and_then(|stages| stages.get_mut(stage as i64 as usize))
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: map_idx as i64, limit })?;

            *record = value as i16;

            Ok(())
        }
        0x07 => {
            let maps = &mut ctx.stage_record_neg19;

            if (maps.len() / 0x31) as u64 <= map_idx as i64 as u64 {
                return Err(Fault::IndexOutOfRange { site: SITE, index: map_idx as i64, limit: (maps.len() / 0x31) as i64 });
            }

            if stage as u32 >= 0x31 || star != 0 {
                return Err(Fault::IndexOutOfRange { site: SITE, index: stage as i64, limit: 0x31 });
            }

            let cell = (map_idx as i64) * 0x31 + stage as u32 as i64;
            let limit = maps.len() as i64;
            let record = maps.get_mut(cell as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: cell, limit })?;

            *record = value as i16;

            Ok(())
        }
        0x06 | 0x08..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (&mut ctx.stage_record_neg20, (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64),
                0x08 => (&mut ctx.stage_record_neg18, (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64),
                0x09 => (&mut ctx.stage_record_neg17, (map_idx as i64) * 8 + stage as i64 + star as i64),
                0x0a => (&mut ctx.stage_record_neg16, (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64),
                _ => (&mut ctx.stage_record_neg11, (map_idx as i64) * 0xc0 + (stage as i64) * 4 + star as i64),
            };
            let limit = maps.len() as i64;
            let record = maps.get_mut(cell as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: cell, limit })?;

            *record = value as i16;

            Ok(())
        }
        0x10 | 0x11 | 0x16 => {
            let (maps, cell) = match case {
                0x10 => (&mut ctx.stage_record_neg10, (map_idx as i64) * 0x3c + (stage as i64) * 4 + star as i64),
                0x11 => (&mut ctx.stage_record_neg9, (map_idx as i64) * 0x30 + (stage as i64) * 4 + star as i64),
                _ => (&mut ctx.stage_record_neg4, (map_idx as i64) * 0xc8 + (stage as i64) * 4 + star as i64),
            };
            let limit = maps.len() as i64;
            let record = maps.get_mut(cell as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: cell, limit })?;

            *record = value;

            Ok(())
        }
        0x13 | 0x17 | 0x18 => {
            let chapter = match case {
                0x13 => map_idx.wrapping_add(7),
                0x17 => map_idx.wrapping_add(4),
                _ => map_idx,
            };
            let row = (chapter as i64 as usize).wrapping_mul(0xd0).wrapping_add(AppContext::STAGE_RECORD_CHAPTERS);
            let key = ctx.i32_at(row.wrapping_add(0xcc))?;

            ctx.set_i32_at(row.wrapping_add((stage as i64 as usize).wrapping_mul(4)), value ^ key)
        }
        0x14 => {
            let cell = (map_idx as i64) * 0x320 + (stage as i64) * 0x10 + (star as i64) * 4 + AppContext::STAGE_RECORD_NEG6 as i64;

            ctx.set_i32_at(cell as usize, value)
        }
        _ => Ok(()),
    }
}
