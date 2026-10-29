use crate::Fault;

use super::AppContext;

pub fn show_rewarded_ad(ctx: &mut AppContext, kind: i32) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .show_rewarded_ad(kind);

    Ok(())
}
