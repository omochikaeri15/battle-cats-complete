use crate::Fault;

use super::{AppContext, get_score_time_limit};

pub fn set_point_stage(ctx: &mut AppContext, stage: i32) -> Result<(), Fault> {
    ctx.event_items
        .as_mut()
        .ok_or(Fault::NullPointer {
            site: "set_point_stage",
        })?
        .rule_id = stage;

    let limit = get_score_time_limit(ctx)?;

    ctx.event_items
        .as_mut()
        .ok_or(Fault::NullPointer {
            site: "set_point_stage",
        })?
        .progress_cap = if limit >= 0 { limit } else { -1 };

    Ok(())
}
