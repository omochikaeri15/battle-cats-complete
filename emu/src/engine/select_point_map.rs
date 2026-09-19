use crate::Fault;

use super::AppContext;

pub fn select_point_map(ctx: &mut AppContext, map_id: i32) -> Result<(), Fault> {
    let store = ctx.event_items.as_mut().ok_or(Fault::NullPointer { site: "select_point_map" })?;

    if !store.rule_tables.contains_key(&map_id) {
        store.score_stage = false;

        return Ok(());
    }

    store.score_stage = true;
    store.stage_key = map_id;
    store.rules = Some(store.rule_tables.get(&map_id).cloned().ok_or(Fault::KeyNotFound { site: "select_point_map", key: map_id as i64 })?);

    Ok(())
}
