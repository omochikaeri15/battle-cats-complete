use crate::{operation, Fault};

use super::{get_stage_count, get_stage_record, get_user_rank, is_map_cleared, map_index_of_map_id, map_type_of_map_id, unlock_group_met, AppContext};

pub fn unlock_condition_met(ctx: &mut AppContext, id: i32) -> Result<bool, Fault> {
    const SITE: &str = "unlock_condition_met";

    let slot = match id {
        -1 => return Ok(true),
        0 => return Ok(false),
        1 => 0usize,
        2 => 1,
        3 => 2,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        20 => {
            let last = get_stage_count(ctx, 0, 0x30)?.wrapping_sub(1);

            return Ok(get_stage_record(ctx, 0, 0x30, last, 0, 0)? > 0);
        }
        25 => return Ok(get_user_rank(ctx)? >= 0x640),
        26 => return Ok(is_map_cleared(ctx, 1, 0xd, 0, 0)? && is_map_cleared(ctx, 1, 0x1a, 0, 0)? && is_map_cleared(ctx, 1, 0x1d, 0, 0)?),
        27 => return Ok(is_map_cleared(ctx, 1, 0x1e, 0, 0)? && is_map_cleared(ctx, 1, 0x1f, 0, 0)? && is_map_cleared(ctx, 1, 0x20, 0, 0)?),
        28 => return Ok(is_map_cleared(ctx, 1, 9, 0, 0)? && is_map_cleared(ctx, 1, 0x24, 0, 0)? && is_map_cleared(ctx, 1, 0x26, 0, 0)?),
        29 => return Ok(is_map_cleared(ctx, 1, 0xa, 0, 0)? && is_map_cleared(ctx, 1, 0xb, 0, 0)? && is_map_cleared(ctx, 1, 0xc, 0, 0)?),
        _ => {
            if id == 0xf423f {
                return Ok(false);
            }

            let map_id = id.wrapping_sub(0x186a0);

            if map_id as u32 <= 0x1869f {
                return is_map_cleared(ctx, map_type_of_map_id(map_id), map_index_of_map_id(map_id), 0, 0);
            }

            let listed = id.wrapping_sub(0x30d40);

            if listed as u32 <= 0x1869f {
                return Ok(ctx.condition_list_200k.contains(&listed));
            }

            let listed = id.wrapping_sub(0x493e0);

            if listed as u32 <= 0x1869f {
                return Ok(ctx.condition_list_300k.contains(&listed));
            }

            return unlock_group_met(ctx, id);
        }
    };
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS.wrapping_add(slot.wrapping_mul(4)))?);
    pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::CHAPTER_PROGRESS_KEY)?);

    Ok(operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 1 })? as i32 >= 0x30)
}
