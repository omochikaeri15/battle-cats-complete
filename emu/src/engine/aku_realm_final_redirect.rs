use crate::Fault;

use super::{get_map_type, get_stage_count, get_stage_index, get_stage_record, AppContext};

pub fn aku_realm_final_redirect(ctx: &mut AppContext, in_stage: u8) -> Result<bool, Fault> {
    if get_map_type(ctx, 0)? != -19 {
        return Ok(false);
    }

    if get_stage_record(ctx, -8, 0x2a, 0, 0, 0)? != 0 {
        return Ok(false);
    }

    if get_stage_record(ctx, -8, 0x2a, 0, 0, 0)? > 0 {
        return Ok(false);
    }

    if in_stage != 0 && get_stage_index(ctx)? != 0x1d {
        return Ok(false);
    }

    let mut stage = 0i32;

    if get_stage_count(ctx, -19, 0)? <= 0 {
        return Ok(true);
    }

    loop {
        let cleared = get_stage_record(ctx, -19, 0, stage, 0, 0)? != 0;

        if !cleared {
            return Ok(cleared);
        }

        stage += 1;

        if stage >= get_stage_count(ctx, -19, 0)? {
            return Ok(cleared);
        }
    }
}
