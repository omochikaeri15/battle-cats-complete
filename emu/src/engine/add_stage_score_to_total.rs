use crate::Fault;

use super::{set_point_total, AppContext};

pub fn add_stage_score_to_total(ctx: &mut AppContext) -> Result<(), Fault> {
    let store = ctx.event_items.as_ref().ok_or(Fault::NullPointer { site: "add_stage_score_to_total" })?;
    let Some(point_id) = store.stage_points.get(&store.stage_key).copied() else {
        return Ok(());
    };
    let value = store.records.get(&point_id).map_or(0, |record| record.total).wrapping_add(store.total);
    let value = if value >= store.point_cap { store.point_cap } else { value };

    set_point_total(ctx, point_id, value)
}
