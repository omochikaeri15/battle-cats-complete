use crate::Fault;

use super::AppContext;

pub fn medal_popup_update(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::host_missing())?.medal_popup_update();

    Ok(())
}
