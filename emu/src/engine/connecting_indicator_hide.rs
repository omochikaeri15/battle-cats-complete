use crate::Fault;

use super::AppContext;

pub fn connecting_indicator_hide(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .connecting_hide();

    Ok(())
}
