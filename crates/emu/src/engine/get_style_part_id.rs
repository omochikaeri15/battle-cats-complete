use crate::Fault;

use super::{
    AppContext, get_built_deck_cannon, get_built_deck_stage_key, get_preset_style_part,
    has_built_deck, has_fixed_lineup,
};

pub fn get_style_part_id(ctx: &mut AppContext) -> Result<i32, Fault> {
    if has_fixed_lineup(ctx, -1, -1, -1)? && ctx.i32_at(AppContext::LINEUP_CANNON_TYPE)? != -1 {
        return ctx.i32_at(AppContext::LINEUP_CANNON_TYPE);
    }

    if ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 {
        let stage_key = get_built_deck_stage_key(ctx)?;

        if has_built_deck(ctx, stage_key)? {
            return Ok(((get_built_deck_cannon(ctx, stage_key)? as u32 >> 8) as u8 as i8) as i32);
        }
    }

    get_preset_style_part(ctx)
}
