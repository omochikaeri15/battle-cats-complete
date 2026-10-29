use crate::Fault;

use super::AppContext;

pub fn notice_popup_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::host_missing())?.notice_popup_draw();

    Ok(())
}
