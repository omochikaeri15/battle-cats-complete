use crate::Fault;

use super::{get_stage_record, get_star_level, is_map_cleared, validate_map_type, AppContext};

pub fn ex_redirect_check_b(ctx: &mut AppContext) -> Result<bool, Fault> {
    if validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?) != -9 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::MAP_INDEX)? != 0x30 {
        return Ok(false);
    }

    if get_star_level(ctx)? != 3 {
        return Ok(false);
    }

    if !is_map_cleared(ctx, -9, 0x30, 3, 0)? {
        return Ok(false);
    }

    Ok(get_stage_record(ctx, -8, 0x45, 0, 0, 0)? == 0)
}
