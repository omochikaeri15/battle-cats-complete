use crate::Fault;

use super::{AppContext, get_cleared_count, get_stage_count, map_type_base_id, min_i32};

pub fn get_stage_unlock(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    star: i32,
    use_cache: i32,
) -> Result<i32, Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);

        if !ctx.stage_unlock_cache.contains_key(&map_id) {
            return Ok(0);
        }

        return ctx
            .stage_unlock_cache
            .entry(map_id)
            .or_default()
            .get(star as i64 as usize)
            .map(|cell| *cell as i32)
            .ok_or(Fault::index_out_of_range(star as i64, 4));
    }

    if map_type as u32 <= 4 {
        let cell = (map_type as u32 as i64) * 0x7d0
            + (map_idx as i64) * 4
            + star as i64
            + AppContext::STAGE_UNLOCK_STORY as i64;

        return Ok(ctx.i8_at(cell as usize)? as i32);
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &ctx.stage_unlock_neg26,
                0x02 => &ctx.stage_unlock_neg24,
                0x03 => &ctx.stage_unlock_neg23,
                _ => &ctx.stage_unlock_neg22,
            };

            maps.get(map_idx as i64 as usize)
                .and_then(|stars| stars.get(star as i64 as usize))
                .map(|cell| *cell as i32)
                .ok_or(Fault::index_out_of_range(map_idx as i64, maps.len() as i64))
        }
        0x05 => {
            let cleared = get_cleared_count(ctx, AppContext::LABYRINTH)?;
            let last = get_stage_count(ctx, -0x15, map_idx)?.wrapping_sub(1);

            Ok(min_i32(cleared, last))
        }
        0x06..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (&ctx.stage_unlock_neg20, (map_idx as i64) * 4 + star as i64),
                0x07 => (&ctx.stage_unlock_neg19, map_idx as i64 + star as i64),
                0x08 => (&ctx.stage_unlock_neg18, (map_idx as i64) * 4 + star as i64),
                0x09 => (&ctx.stage_unlock_neg17, map_idx as i64 + star as i64),
                0x0a => (&ctx.stage_unlock_neg16, (map_idx as i64) * 4 + star as i64),
                _ => (&ctx.stage_unlock_neg11, (map_idx as i64) * 4 + star as i64),
            };

            maps.get(cell as usize)
                .map(|value| *value as i32)
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x10 | 0x11 | 0x16 => {
            let maps = match case {
                0x10 => &ctx.stage_unlock_neg10,
                0x11 => &ctx.stage_unlock_neg9,
                _ => &ctx.stage_unlock_neg4,
            };
            let cell = (map_idx as i64) * 4 + star as i64;

            maps.get(cell as usize)
                .copied()
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x14 => {
            let cell =
                (map_idx as i64) * 0x10 + (star as i64) * 4 + AppContext::STAGE_UNLOCK_NEG6 as i64;

            ctx.i32_at(cell as usize)
        }
        0x13 | 0x17 | 0x18 => {
            let base = match case {
                0x13 => AppContext::STAGE_UNLOCK_NEG7,
                0x17 => AppContext::STAGE_UNLOCK_NEG3,
                _ => AppContext::STAGE_UNLOCK_CHAPTERS,
            };

            ctx.i32_at(((map_idx as i64) * 4 + base as i64) as usize)
        }
        _ => Ok(0),
    }
}
