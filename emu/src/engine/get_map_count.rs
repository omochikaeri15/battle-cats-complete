use crate::Fault;

use super::AppContext;

pub fn get_map_count(ctx: &AppContext, map_type: i32) -> Result<i32, Fault> {
    let case = map_type.wrapping_add(0x1a) as u32;

    if case > 0x1e {
        return Ok(0);
    }

    let maps = match case {
        0x00 => &ctx.stage_table_neg26.names,
        0x01 | 0x0b | 0x14 => return Ok(1),
        0x02 => &ctx.stage_table_neg24.names,
        0x03 => &ctx.stage_table_neg23.names,
        0x04 => &ctx.stage_table_neg22.names,
        0x05 => &ctx.stage_name_variants[16],
        0x06 => &ctx.stage_name_variants[15],
        0x07 => &ctx.stage_name_variants[14],
        0x08 => &ctx.stage_name_variants[13],
        0x09 => &ctx.stage_name_variants[12],
        0x0a => &ctx.stage_name_variants[11],
        0x0c | 0x0d | 0x0e | 0x13 | 0x17 | 0x18 => return Ok(3),
        0x0f => &ctx.stage_name_variants[10],
        0x10 => &ctx.stage_name_variants[9],
        0x11 => &ctx.stage_name_variants[8],
        0x12 => return Ok(0x52),
        0x15 => return Ok(0x10),
        0x16 => &ctx.stage_name_variants[3],
        0x1a..=0x1e => {
            return ctx
                .i32_at(AppContext::STORY_MAP_COUNTS.wrapping_add((case as usize - 0x1a) * 4));
        }
        _ => return Ok(0),
    };

    Ok(((maps.len() * 3) as u32).wrapping_mul(0xaaaa_aaab) as i32)
}
