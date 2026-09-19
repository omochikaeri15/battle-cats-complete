use crate::Fault;

use super::AppContext;

pub fn reset_point_state(ctx: &mut AppContext) -> Result<(), Fault> {
    let store = ctx.event_items.as_mut().ok_or(Fault::NullPointer { site: "reset_point_state" })?;

    store.progress = 0;
    store.progress_cap = 0;
    store.total = 0;

    Ok(())
}
