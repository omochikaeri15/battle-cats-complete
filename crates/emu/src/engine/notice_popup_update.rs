use crate::Fault;

use super::AppContext;

pub fn notice_popup_update(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx.meta().ok_or(Fault::host_missing())?.notice_popup_update())
}
