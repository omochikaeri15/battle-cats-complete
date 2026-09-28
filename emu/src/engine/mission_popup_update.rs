use crate::Fault;

use super::AppContext;

pub fn mission_popup_update(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::host_missing())?.mission_popup_update();

    Ok(())
}
