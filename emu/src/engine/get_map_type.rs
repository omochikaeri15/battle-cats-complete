use crate::Fault;

use super::{
    AppContext, ex_redirect_check_a, ex_redirect_check_b, get_scene_id, is_invasion_stage,
    is_outbreak_stage, is_z_invasion_stage,
};

pub fn get_map_type(ctx: &mut AppContext, base_only: u8) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3 {
        if (ex_redirect_check_a(ctx)? || ex_redirect_check_b(ctx)?)
            && get_scene_id(ctx)? != 0x12c
            && base_only == 0
        {
            return Ok(-8);
        }

        let saved_type = ctx.i32_at(AppContext::SAVED_MAP_TYPE)?;

        return Ok(if (saved_type.wrapping_add(0x1a) as u32) < 0x1f {
            saved_type
        } else {
            -1
        });
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? >= 0 && ctx.i32_at(AppContext::CHAPTER_MODE)? <= 2 {
        let outbreak = is_outbreak_stage(ctx)?;
        let variant = if base_only != 0 { -2 } else { -0xc };

        return Ok(if outbreak { variant } else { -2 });
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? >= 4 && ctx.i32_at(AppContext::CHAPTER_MODE)? <= 6 {
        let outbreak = is_outbreak_stage(ctx)?;
        let variant = if base_only != 0 { -3 } else { -0xd };

        return Ok(if outbreak { variant } else { -3 });
    }

    if ctx.i32_at(AppContext::CHAPTER_MODE)? >= 7 && ctx.i32_at(AppContext::CHAPTER_MODE)? <= 9 {
        if is_invasion_stage(ctx)? && base_only == 0 {
            return Ok(-0xf);
        }

        if is_z_invasion_stage(ctx)? && base_only == 0 {
            return Ok(-0x19);
        }

        let outbreak = is_outbreak_stage(ctx)?;
        let variant = if base_only != 0 { -7 } else { -0xe };

        return Ok(if outbreak { variant } else { -7 });
    }

    Ok(if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63 {
        -1
    } else {
        -8
    })
}
