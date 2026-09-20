use crate::{Fault, ops};

use super::{
    AppContext, Entity, get_castle_id, get_castle_row, get_entity_button, get_unit_model,
    mamodel_get_anchor, mamodel_get_anchor_part, mamodel_get_anchor_x, mamodel_get_anchor_y,
    read_flag, transform_anchor,
};

pub fn compute_back_bound(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    let button = get_entity_button(ctx, faction, slot)?;
    let idx = read_flag(ctx, AppContext::faction_flags(faction))? & 1;

    let (side, index) =
        get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;
    let model = &ctx.unit_models[side][index];
    let part_index = mamodel_get_anchor_part(mamodel_get_anchor(model, idx)?);
    let part = model
        .parts
        .get(part_index as i64 as usize)
        .ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;
    let (side, index) =
        get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;
    let model = &ctx.unit_models[side][index];
    let (side, index) =
        get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;
    let x = mamodel_get_anchor_x(mamodel_get_anchor(&ctx.unit_models[side][index], idx)?);
    let (side, index) =
        get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;
    let y = mamodel_get_anchor_y(mamodel_get_anchor(&ctx.unit_models[side][index], idx)?);
    let mut out = 0i64;

    transform_anchor(part, model, x, y, &mut out)?;

    let reach = out as i32;

    if faction == 0 {
        let bound = ctx
            .i32_at(AppContext::STAGE_LENGTH)?
            .wrapping_sub((reach << 2).wrapping_mul(5));

        return ctx.set_i32_at(
            AppContext::entity_field(faction, slot, Entity::BACK_BOUND),
            bound,
        );
    }

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::BACK_BOUND),
        reach.wrapping_add(reach).wrapping_mul(5),
    )?;

    if faction != 1 {
        return Ok(());
    }

    if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
        return Ok(());
    }

    if ctx.i32_at(AppContext::entity_field(1, slot, Entity::BOSS_TYPE))? == 0 {
        return Ok(());
    }

    let base_x = ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_X))?;
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
    let inset =
        (ops::div_neg_100(size.wrapping_mul(0x49c) as i64) as i32).wrapping_add(base_x);
    let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
    let shifted =
        (ops::div_10(size.wrapping_mul(offset_x) as i64) as i32).wrapping_add(inset);
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
    let width = ops::div_10((size << 7).wrapping_sub(size) as i64) as i32;
    let bound = shifted
        .wrapping_add(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BACK_BOUND))?)
        .wrapping_add(width);

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::BACK_BOUND),
        bound,
    )
}
