use crate::Fault;

use super::{AppContext, get_cleared_count, get_stage_count, map_type_base_id, xor_row_get};

pub fn get_stages_cleared(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    crown: i32,
    use_cache: i32,
) -> Result<i32, Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);

        if !ctx.stages_cleared_cache.contains_key(&map_id) {
            return Ok(0);
        }

        return ctx
            .stages_cleared_cache
            .entry(map_id)
            .or_default()
            .get(crown as usize)
            .map(|cleared| *cleared as i32)
            .ok_or(Fault::index_out_of_range(crown as i64, 4));
    }

    let map_id = map_type_base_id(map_type, map_idx);

    if ctx.cleared_map_ids.contains(&map_id) {
        return get_stage_count(ctx, map_type, map_idx);
    }

    if map_type as u32 <= 4 {
        let cell = (map_type as u32 as i64) * 0x7d0
            + (map_idx as i64) * 4
            + crown as i64
            + AppContext::STAGES_CLEARED_STORY as i64;

        return Ok(ctx.i8_at(cell as usize)? as i32);
    }

    let case = map_type.wrapping_add(0x18) as u32;

    match case {
        0x00..=0x02 => {
            let maps = match case {
                0x00 => &ctx.stages_cleared_neg24,
                0x01 => &ctx.stages_cleared_neg23,
                _ => &ctx.stages_cleared_neg22,
            };

            maps.get(map_idx as usize)
                .and_then(|crowns| crowns.get(crown as usize))
                .map(|cleared| *cleared as i32)
                .ok_or(Fault::index_out_of_range(map_idx as i64, maps.len() as i64))
        }
        0x03 => get_cleared_count(ctx, 0xad0),
        0x05 => Ok(0x31),
        0x04 | 0x06..=0x08 | 0x0d => {
            let (maps, cell) = match case {
                0x04 => (
                    &ctx.stages_cleared_neg20,
                    (map_idx as i64) * 4 + crown as i64,
                ),
                0x06 => (
                    &ctx.stages_cleared_neg18,
                    (map_idx as i64) * 4 + crown as i64,
                ),
                0x07 => (&ctx.stages_cleared_neg17, map_idx as i64 + crown as i64),
                0x08 => (
                    &ctx.stages_cleared_neg16,
                    (map_idx as i64) * 4 + crown as i64,
                ),
                _ => (
                    &ctx.stages_cleared_neg11,
                    (map_idx as i64) * 4 + crown as i64,
                ),
            };

            maps.get(cell as usize)
                .map(|cleared| *cleared as i32)
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x0e | 0x0f | 0x14 => {
            let maps = match case {
                0x0e => &ctx.stages_cleared_neg10,
                0x0f => &ctx.stages_cleared_neg9,
                _ => &ctx.stages_cleared_neg4,
            };
            let cell = (map_idx as i64) * 4 + crown as i64;

            maps.get(cell as usize)
                .copied()
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x12 => {
            let cell = (map_idx as i64) * 0x10
                + (crown as i64) * 4
                + AppContext::STAGES_CLEARED_NEG6 as i64;

            ctx.i32_at(cell as usize)
        }
        0x11 | 0x15 | 0x16 => {
            let chapter = match case {
                0x11 => map_idx.wrapping_add(7),
                0x15 => map_idx.wrapping_add(4),
                _ => map_idx,
            };

            xor_row_get(
                ctx.bytes_from(AppContext::STAGES_CLEARED_CHAPTERS)?,
                chapter as i64 as usize,
            )
            .map(|cleared| cleared as i32)
            .ok_or(Fault::index_out_of_range(chapter as i64, 10))
        }
        _ => Ok(0),
    }
}
