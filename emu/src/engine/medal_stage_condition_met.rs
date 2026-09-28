use crate::{Fault, ops};

use super::{AppContext, Medal, get_stage_count, get_stage_record, map_index_of_map_id, map_type_of_map_id};

pub fn medal_stage_condition_met(ctx: &mut AppContext, medal: &Medal, map_id: i32) -> Result<bool, Fault> {
    let map_type = map_type_of_map_id(map_id);
    let mut map_idx = map_index_of_map_id(map_id);
    let mut stage = medal.stage;

    if stage == -1 {
        stage = get_stage_count(ctx, map_type, map_idx)?.wrapping_sub(1);
    }

    let mut met = false;

    if get_stage_record(ctx, map_type, map_idx, stage, medal.crown, 0)? != 0 {
        met = true;

        if medal.treasure != -1 && medal.stage == -1 {
            if map_type == -7 {
                map_idx = map_idx.wrapping_add(7);
            } else if map_type == -2 {
            } else if map_type == -3 {
                map_idx = map_idx.wrapping_add(4);
            } else {
                return Ok(false);
            }

            let mut castle = 0usize;

            while castle != 0x30 {
                let row = ctx.bytes_from(
                    AppContext::TREASURE_LEVELS + map_idx as i64 as usize * AppContext::TREASURE_LEVELS_STRIDE,
                )?;
                let level =
                    ops::xor_row_decode(row, 0x31, castle).ok_or(Fault::index_out_of_range(castle as i64, 0x31))? as i32;

                met = level > medal.treasure;

                if level <= medal.treasure {
                    break;
                }

                castle += 1;
            }
        }
    }

    Ok(met)
}
