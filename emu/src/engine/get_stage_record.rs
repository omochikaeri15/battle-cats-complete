use crate::Fault;

use super::{
    AppContext, get_cleared_count, map_type_as_index, map_type_base_id, std_map_int_bool_subscript,
    std_map_int_map_subscript, std_map_int_map_subscript_2, xor_row51_get,
};

pub fn get_stage_record(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    stage: i32,
    star: i32,
    use_cache: i32,
) -> Result<i32, Fault> {
    if use_cache != 0 {
        let map_id = map_type_base_id(map_type, map_idx);

        return ctx
            .stage_record_cache
            .entry(map_id)
            .or_default()
            .entry(stage)
            .or_default()
            .get(star as usize)
            .map(|record| *record as i32)
            .ok_or(Fault::index_out_of_range(star as i64, 4));
    }

    if map_type as u32 <= 4 {
        let cell = (map_type_as_index(map_type) as i64) * 0xbb80
            + (map_idx as i64) * 0x60
            + (stage as i64) * 8
            + (star as i64) * 2
            + AppContext::STAGE_RECORD_STORY as i64;

        return Ok(ctx.i16_at(cell as usize)? as i32);
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &ctx.stage_record_neg26,
                0x02 => &ctx.stage_record_neg24,
                0x03 => &ctx.stage_record_neg23,
                _ => &ctx.stage_record_neg22,
            };

            maps.get(map_idx as usize)
                .and_then(|stars| stars.get(star as usize))
                .and_then(|stages| stages.get(stage as usize))
                .map(|record| *record as i32)
                .ok_or(Fault::index_out_of_range(map_idx as i64, maps.len() as i64))
        }
        0x01 => Ok(ctx.u8_at(AppContext::MAP_NEG25_CLEARED)? as i32),
        0x05 => Ok((get_cleared_count(ctx, 0xad0)? > stage) as i32),
        0x07 => {
            let maps = &ctx.stage_record_neg19;

            if (maps.len() / 0x31) as u64 <= map_idx as i64 as u64 {
                return Err(Fault::index_out_of_range(map_idx as i64, (maps.len() / 0x31) as i64));
            }

            if stage as u32 >= 0x31 || star != 0 {
                return Err(Fault::index_out_of_range(stage as i64, 0x31));
            }

            let cell = (map_idx as i64) * 0x31 + stage as u32 as i64;

            maps.get(cell as usize)
                .map(|record| *record as i32)
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x06 | 0x08..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (
                    &ctx.stage_record_neg20,
                    (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64,
                ),
                0x08 => (
                    &ctx.stage_record_neg18,
                    (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64,
                ),
                0x09 => (
                    &ctx.stage_record_neg17,
                    (map_idx as i64) * 8 + stage as i64 + star as i64,
                ),
                0x0a => (
                    &ctx.stage_record_neg16,
                    (map_idx as i64) * 0x78 + (stage as i64) * 4 + star as i64,
                ),
                _ => (
                    &ctx.stage_record_neg11,
                    (map_idx as i64) * 0xc0 + (stage as i64) * 4 + star as i64,
                ),
            };

            maps.get(cell as usize)
                .map(|record| *record as i32)
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x0b => Ok(ctx.u8_at(AppContext::MAP_NEG15_CLEARED)? as i32),
        0x0c..=0x0e => {
            let stages = match case {
                0x0c => {
                    std_map_int_map_subscript(&mut ctx.outbreak_cleared, &map_idx.wrapping_add(7))
                }
                0x0d => {
                    std_map_int_map_subscript(&mut ctx.outbreak_cleared, &map_idx.wrapping_add(4))
                }
                _ => std_map_int_map_subscript_2(&mut ctx.outbreak_cleared, &map_idx),
            };

            Ok(*std_map_int_bool_subscript(stages, &stage) as i32)
        }
        0x10 | 0x11 | 0x16 => {
            let (maps, cell) = match case {
                0x10 => (
                    &ctx.stage_record_neg10,
                    (map_idx as i64) * 0x3c + (stage as i64) * 4 + star as i64,
                ),
                0x11 => (
                    &ctx.stage_record_neg9,
                    (map_idx as i64) * 0x30 + (stage as i64) * 4 + star as i64,
                ),
                _ => (
                    &ctx.stage_record_neg4,
                    (map_idx as i64) * 0xc8 + (stage as i64) * 4 + star as i64,
                ),
            };

            maps.get(cell as usize)
                .copied()
                .ok_or(Fault::index_out_of_range(cell, maps.len() as i64))
        }
        0x12 => {
            let cell =
                (map_idx as i64) * 0x30 + (stage as i64) * 4 + AppContext::STAGE_RECORD_NEG8 as i64;

            ctx.i32_at(cell as usize)
        }
        0x13 | 0x17 | 0x18 => {
            let chapter = match case {
                0x13 => map_idx.wrapping_add(7),
                0x17 => map_idx.wrapping_add(4),
                _ => map_idx,
            };
            let row = (chapter as i64) * 0xd0 + AppContext::STAGE_RECORD_CHAPTERS as i64;

            xor_row51_get(ctx.bytes_from(row as usize)?, stage as i64 as usize)
                .map(|record| record as i32)
                .ok_or(Fault::index_out_of_range(stage as i64, 0x33))
        }
        0x14 => {
            let cell = (map_idx as i64) * 0x320
                + (stage as i64) * 0x10
                + (star as i64) * 4
                + AppContext::STAGE_RECORD_NEG6 as i64;

            ctx.i32_at(cell as usize)
        }
        _ => Ok(0),
    }
}
