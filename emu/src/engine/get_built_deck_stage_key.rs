use crate::Fault;

use super::{
    aku_realm_final_redirect, ex_redirect_check_c, ex_replacement_pending, get_ex_option_target, get_global_map_id, get_map_type, get_stage_index,
    get_star_level, is_ex_option_target, map_type_base_id, AppContext,
};

pub fn get_built_deck_stage_key(ctx: &mut AppContext) -> Result<i32, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;
    let stage = get_stage_index(ctx)?;
    let star_base = if get_star_level(ctx)? != 3 { 0 } else { 1_000_000_000i32 };

    if ex_replacement_pending(ctx, -1, -1)? {
        return Ok(star_base.wrapping_add(stage).wrapping_add(get_ex_option_target(ctx, map_id).wrapping_mul(1000)));
    }

    if is_ex_option_target(ctx)? {
        return Ok(stage.wrapping_add(star_base).wrapping_add(map_id.wrapping_mul(1000)));
    }

    if get_map_type(ctx, 0)? == -8 {
        return ctx.i32_at(AppContext::BUILT_DECK_EX_STAGE_KEY);
    }

    if aku_realm_final_redirect(ctx, 1)? {
        return Ok(map_type_base_id(-8, 0x2a).wrapping_mul(1000));
    }

    if ex_redirect_check_c(ctx, 1)? {
        return Ok(map_type_base_id(-8, 0x44).wrapping_mul(1000));
    }

    Ok(stage.wrapping_add(star_base).wrapping_add(map_id.wrapping_mul(1000)))
}
