use crate::Fault;

use super::{AppContext, set_point_total};

pub fn add_stage_score_to_total(ctx: &mut AppContext) -> Result<(), Fault> {
    let store = ctx.event_items.as_ref().ok_or(Fault::null_pointer())?;
    let Some(point_id) = store.point_id_by_map.get(&store.stage_key).copied() else {
        return Ok(());
    };
    let value = store
        .records
        .get(&point_id)
        .map_or(0, |record| record.total)
        .wrapping_add(store.total);
    let value = if value >= store.point_cap {
        store.point_cap
    } else {
        value
    };

    set_point_total(ctx, point_id, value)
}
