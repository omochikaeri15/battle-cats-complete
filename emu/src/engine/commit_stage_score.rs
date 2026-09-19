use crate::Fault;

use super::{add_stage_score_to_total, set_stage_best_score, AppContext};

pub fn commit_stage_score(ctx: &mut AppContext) -> Result<(), Fault> {
    add_stage_score_to_total(ctx)?;

    let store = ctx.event_items.as_mut().ok_or(Fault::NullPointer { site: "commit_stage_score" })?;
    let Some(point_id) = store.stage_points.get(&store.stage_key).copied() else {
        return Ok(());
    };
    let (key, stage, total) = (store.stage_key, store.rule_id, store.total);

    set_stage_best_score(store, point_id, key, stage, total);

    Ok(())
}
