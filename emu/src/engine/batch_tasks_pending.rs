use crate::Fault;

use super::AppContext;

pub fn batch_tasks_pending(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx.scene_host().ok_or(Fault::host_missing())?.loader_busy())
}
