use crate::Fault;

use super::{AppContext, get_map_index, get_map_type, is_aku_final_map, is_ex_option_target};

pub fn ex_stage_credits_gauge(ctx: &mut AppContext) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63 {
        return Ok(false);
    }

    if get_map_type(ctx, 0)? == -8 && get_map_index(ctx, 0)? == 0x44 {
        return Ok(true);
    }

    if is_aku_final_map(ctx)? {
        return Ok(true);
    }

    if is_ex_option_target(ctx)? {
        return Ok(true);
    }

    let listed = [0x1ai32, 0x45];

    if get_map_type(ctx, 0)? == -8 {
        let map_index = get_map_index(ctx, 0)?;

        return Ok(listed.contains(&map_index));
    }

    Ok(false)
}
