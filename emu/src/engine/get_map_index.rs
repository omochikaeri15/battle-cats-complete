use crate::Fault;

use super::{get_map_type, AppContext};

pub fn get_map_index(ctx: &mut AppContext, base_only: u8) -> Result<i32, Fault> {
    if get_map_type(ctx, 0)? == -2 || get_map_type(ctx, 0)? == -0xc {
        return ctx.i32_at(AppContext::CHAPTER_MODE);
    }

    if get_map_type(ctx, 0)? == -3 || get_map_type(ctx, 0)? == -0xd {
        return Ok(ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_add(-4));
    }

    if get_map_type(ctx, 0)? == -7 || get_map_type(ctx, 0)? == -0xe {
        return Ok(ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_add(-7));
    }

    if get_map_type(ctx, 0)? == -8 {
        return ctx.i32_at(AppContext::EX_MAP_INDEX);
    }

    if get_map_type(ctx, 0)? == -0xf || get_map_type(ctx, 0)? == -0x19 {
        return Ok((base_only as i32).wrapping_add(base_only as i32));
    }

    ctx.i32_at(AppContext::MAP_INDEX)
}
