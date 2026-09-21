use crate::Fault;

use super::{
    AppContext, get_map_count, get_map_index, get_map_type, get_stage_index, get_stage_record,
    get_crown_level, is_map_cleared,
};

pub fn ex_redirect_check_c(ctx: &mut AppContext, in_stage: u8) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? != -9 {
        return Ok(false);
    }

    if get_map_index(ctx, 0)? != 0x23 {
        return Ok(false);
    }

    if get_crown_level(ctx)? != 0 {
        return Ok(false);
    }

    if in_stage != 0 {
        if get_stage_index(ctx)? != 5 {
            return Ok(false);
        }

        if ctx.u8_at(AppContext::EX_REDIRECT_A_BLOCKED)? == 0 {
            return Ok(false);
        }
    } else if ctx.u8_at(AppContext::EX_REDIRECT_A_BLOCKED)? == 0 {
        return Ok(false);
    }

    if get_stage_record(ctx, -8, 0x44, 0, 0, 0)? != 0 {
        return Ok(false);
    }

    let mut map_idx = get_map_count(ctx, -9)?;

    loop {
        if map_idx <= 0 {
            return Ok(true);
        }

        map_idx -= 1;

        if !is_map_cleared(ctx, -9, map_idx, 0, 0)? {
            return Ok(false);
        }
    }
}
