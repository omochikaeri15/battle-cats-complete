use crate::Fault;

use super::{AppContext, get_release_point_cap, get_scene_id, max_i32, min_i32};

pub fn set_point_total(ctx: &mut AppContext, point_id: i32, value: i32) -> Result<(), Fault> {
    ctx.event_items
        .as_mut()
        .ok_or(Fault::null_pointer())?
        .records
        .entry(point_id)
        .or_default();

    let cap = if (get_scene_id(ctx)? == 100 && ctx.i32_at(AppContext::SCENE_0X64_PAGE)? == 9)
        || get_scene_id(ctx)? == 300
    {
        ctx.event_items
            .as_ref()
            .ok_or(Fault::null_pointer())?
            .point_cap
    } else {
        get_release_point_cap(ctx, point_id)?
    };
    let capped = min_i32(value, cap);
    let current = ctx
        .event_items
        .as_ref()
        .ok_or(Fault::null_pointer())?
        .records
        .get(&point_id)
        .ok_or(Fault::key_not_found(point_id as i64))?
        .total;
    let kept = max_i32(capped, current);
    let record = ctx
        .event_items
        .as_mut()
        .ok_or(Fault::null_pointer())?
        .records
        .get_mut(&point_id)
        .ok_or(Fault::key_not_found(point_id as i64))?;

    record.total = kept;

    Ok(())
}
