use crate::Fault;

use super::{AppContext, map_type_base_id};

const SITE: &str = "set_stage_unlock";

pub fn set_stage_unlock(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    star: i32,
    value: i32,
    use_cache: i32,
) -> Result<(), Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);
        let stars = ctx.stage_unlock_cache.entry(map_id).or_default();
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
            + AppContext::STAGE_UNLOCK_STORY as i64;

        return ctx.set_block_at::<1>(cell as usize, [value as u8]);
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &mut ctx.stage_unlock_neg26,
                0x02 => &mut ctx.stage_unlock_neg24,
                0x03 => &mut ctx.stage_unlock_neg23,
                _ => &mut ctx.stage_unlock_neg22,
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
        0x06..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (
                    &mut ctx.stage_unlock_neg20,
                    (map_idx as i64) * 4 + star as i64,
                ),
                0x07 => (&mut ctx.stage_unlock_neg19, map_idx as i64 + star as i64),
                0x08 => (
                    &mut ctx.stage_unlock_neg18,
                    (map_idx as i64) * 4 + star as i64,
                ),
                0x09 => (&mut ctx.stage_unlock_neg17, map_idx as i64 + star as i64),
                0x0a => (
                    &mut ctx.stage_unlock_neg16,
                    (map_idx as i64) * 4 + star as i64,
                ),
                _ => (
                    &mut ctx.stage_unlock_neg11,
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
                0x10 => &mut ctx.stage_unlock_neg10,
                0x11 => &mut ctx.stage_unlock_neg9,
                _ => &mut ctx.stage_unlock_neg4,
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
            let cell =
                (map_idx as i64) * 0x10 + (star as i64) * 4 + AppContext::STAGE_UNLOCK_NEG6 as i64;

            ctx.set_i32_at(cell as usize, value)
        }
        0x13 | 0x17 | 0x18 => {
            let base = match case {
                0x13 => AppContext::STAGE_UNLOCK_NEG7,
                0x17 => AppContext::STAGE_UNLOCK_NEG3,
                _ => AppContext::STAGE_UNLOCK_CHAPTERS,
            };

            ctx.set_i32_at(((map_idx as i64) * 4 + base as i64) as usize, value)
        }
        _ => Ok(()),
    }
}
