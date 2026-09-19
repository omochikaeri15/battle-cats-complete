use crate::Fault;

use super::AppContext;

pub fn get_map_count(ctx: &AppContext, map_type: i32) -> Result<i32, Fault> {
    let case = map_type.wrapping_add(0x1a) as u32;

    if case > 0x1e {
        return Ok(0);
    }

    let maps = match case {
        0x00 => &ctx.maps_neg26,
        0x01 | 0x0b | 0x14 => return Ok(1),
        0x02 => &ctx.maps_neg24,
        0x03 => &ctx.maps_neg23,
        0x04 => &ctx.maps_neg22,
        0x05 => &ctx.maps_neg21,
        0x06 => &ctx.maps_neg20,
        0x07 => &ctx.maps_neg19,
        0x08 => &ctx.maps_neg18,
        0x09 => &ctx.maps_neg17,
        0x0a => &ctx.maps_neg16,
        0x0c | 0x0d | 0x0e | 0x13 | 0x17 | 0x18 => return Ok(3),
        0x0f => &ctx.maps_neg11,
        0x10 => &ctx.maps_neg10,
        0x11 => &ctx.maps_neg9,
        0x12 => return Ok(0x52),
        0x15 => return Ok(0x10),
        0x16 => &ctx.maps_neg4,
        0x1a..=0x1e => {
            return ctx
                .i32_at(AppContext::STORY_MAP_COUNTS.wrapping_add((case as usize - 0x1a) * 4));
        }
        _ => return Ok(0),
    };

    Ok(((maps.len() * 3) as u32).wrapping_mul(0xaaaa_aaab) as i32)
}
