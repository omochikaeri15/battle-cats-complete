use crate::Fault;

use super::{AppContext, map_type_base_id, set_cleared_count};

const SITE: &str = "set_stages_cleared";

pub fn set_stages_cleared(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    star: i32,
    value: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);
        let stars = ctx.stages_cleared_cache.entry(map_id).or_default();
        let cell = stars
            .get_mut(star as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: star as i64,
                limit: 4,
            })?;

        *cell = value as i16;

        return Ok(());
    }

    if map_type as u32 <= 4 {
        let cell = (map_type as u32 as i64) * 0x7d0
            + (map_idx as i64) * 4
            + star as i64
            + AppContext::STAGES_CLEARED_STORY as i64;

        return ctx.set_block_at::<1>(cell as usize, [value as u8]);
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &mut ctx.stages_cleared_neg26,
                0x02 => &mut ctx.stages_cleared_neg24,
                0x03 => &mut ctx.stages_cleared_neg23,
                _ => &mut ctx.stages_cleared_neg22,
            };
            let limit = maps.len() as i64;
            let cell = maps
                .get_mut(map_idx as i64 as usize)
                .and_then(|stars| stars.get_mut(star as i64 as usize))
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: map_idx as i64,
                    limit,
                })?;

            *cell = value as i8;

            Ok(())
        }
        0x05 => set_cleared_count(ctx, AppContext::LABYRINTH, value),
        0x06 | 0x08..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (
                    &mut ctx.stages_cleared_neg20,
                    (map_idx as i64) * 4 + star as i64,
                ),
                0x08 => (
                    &mut ctx.stages_cleared_neg18,
                    (map_idx as i64) * 4 + star as i64,
                ),
                0x09 => (&mut ctx.stages_cleared_neg17, map_idx as i64 + star as i64),
                0x0a => (
                    &mut ctx.stages_cleared_neg16,
                    (map_idx as i64) * 4 + star as i64,
                ),
                _ => (
                    &mut ctx.stages_cleared_neg11,
                    (map_idx as i64) * 4 + star as i64,
                ),
            };
            let limit = maps.len() as i64;
            let slot = maps.get_mut(cell as usize).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: cell,
                limit,
            })?;

            *slot = value as i8;

            Ok(())
        }
        0x10 | 0x11 | 0x16 => {
            let maps = match case {
                0x10 => &mut ctx.stages_cleared_neg10,
                0x11 => &mut ctx.stages_cleared_neg9,
                _ => &mut ctx.stages_cleared_neg4,
            };
            let cell = (map_idx as i64) * 4 + star as i64;
            let limit = maps.len() as i64;
            let slot = maps.get_mut(cell as usize).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: cell,
                limit,
            })?;

            *slot = value;

            Ok(())
        }
        0x14 => {
            let cell = (map_idx as i64) * 0x10
                + (star as i64) * 4
                + AppContext::STAGES_CLEARED_NEG6 as i64;

            ctx.set_i32_at(cell as usize, value)
        }
        0x13 | 0x17 | 0x18 => {
            let chapter = match case {
                0x13 => map_idx.wrapping_add(7),
                0x17 => map_idx.wrapping_add(4),
                _ => map_idx,
            };
            let key = ctx.i32_at(AppContext::CHAPTER_PROGRESS_KEY)?;
            let cell = (chapter as i64) * 4 + AppContext::STAGES_CLEARED_CHAPTERS as i64;

            ctx.set_i32_at(cell as usize, value ^ key)
        }
        _ => Ok(()),
    }
}
