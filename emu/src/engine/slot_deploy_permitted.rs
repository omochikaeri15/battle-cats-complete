use crate::Fault;

use super::{
    AppContext, deploy_limit_reached, get_built_deck_rows, get_built_deck_stage_key,
    get_button_unit_row, get_current_stage_id, stage_has_restriction, unit_meets_restriction,
};

pub fn slot_deploy_permitted(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    in_battle: u8,
) -> Result<bool, Fault> {
    let mut row = get_button_unit_row(ctx, faction, slot)?;

    if ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 && ctx.i32_at(AppContext::SCENE_0X64_PAGE)? != 3
    {
        let stage_key = get_built_deck_stage_key(ctx)?;
        let rows = get_built_deck_rows(ctx, stage_key)?;

        row = rows
            .get(slot as i64 as usize)
            .ok_or(Fault::index_out_of_range(slot as i64, 10))?
            .0 as i32;
    }

    if faction != 0 || row == -1 {
        return Ok(true);
    }

    let stage_id = get_current_stage_id(ctx)?;

    if stage_has_restriction(ctx, &ctx.stage_restrictions, stage_id)?
        && !unit_meets_restriction(ctx, slot, in_battle)?
    {
        return Ok(false);
    }

    Ok(!deploy_limit_reached(ctx)?)
}
