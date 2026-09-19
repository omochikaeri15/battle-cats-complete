use crate::Fault;

use super::{get_release_point_cap, get_scene_id, AppContext};

const SITE: &str = "set_point_total";

pub fn set_point_total(ctx: &mut AppContext, point_id: i32, value: i32) -> Result<(), Fault> {
    ctx.event_items.as_mut().ok_or(Fault::NullPointer { site: SITE })?.records.entry(point_id).or_default();

    let cap = if (get_scene_id(ctx)? == 100 && ctx.i32_at(AppContext::SCENE_0X64_PAGE)? == 9) || get_scene_id(ctx)? == 300 {
        ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: SITE })?.point_cap
    } else {
        get_release_point_cap(ctx, point_id)?
    };
    let record = ctx
        .event_items
        .as_mut()
        .ok_or(Fault::NullPointer { site: SITE })?
        .records
        .get_mut(&point_id)
        .ok_or(Fault::KeyNotFound { site: SITE, key: point_id as i64 })?;

    record.total = if cap >= value { value } else { cap };

    Ok(())
}
