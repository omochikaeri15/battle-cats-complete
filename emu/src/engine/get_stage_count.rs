use crate::Fault;

use super::{AppContext, get_stage_set_size};

pub fn get_stage_count(ctx: &AppContext, map_type: i32, map_idx: i32) -> Result<i32, Fault> {
    let type_bit = map_type.wrapping_add(0x19) as u32;

    if type_bit <= 0x17 {
        if 0xc43800u32 >> type_bit & 1 != 0 {
            return Ok(0x30);
        }

        if 0x401u32 >> type_bit & 1 != 0 {
            return Ok(1);
        }
    }

    let data_ids = ctx.map_data_ids.get(&map_type).ok_or(Fault::KeyNotFound {
        site: "get_stage_count",
        key: map_type as i64,
    })?;

    if map_idx as u32 >= 0x1f4 {
        return Err(Fault::IndexOutOfRange {
            site: "get_stage_count",
            index: map_idx as i64,
            limit: 0x1f4,
        });
    }

    let data_id = *data_ids
        .get(map_idx as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "get_stage_count",
            index: map_idx as i64,
            limit: 0x1f4,
        })?;

    let map_data = ctx.map_data.get(&data_id).ok_or(Fault::KeyNotFound {
        site: "get_stage_count",
        key: data_id as i64,
    })?;

    let stage_sets = ctx
        .map_stage_sets
        .get(&map_type)
        .ok_or(Fault::KeyNotFound {
            site: "get_stage_count",
            key: map_type as i64,
        })?;

    let set = *stage_sets
        .get(map_idx as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "get_stage_count",
            index: map_idx as i64,
            limit: 0x1f4,
        })?;

    Ok(get_stage_set_size(map_data, set)? as i32)
}
