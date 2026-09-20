use crate::Fault;

use super::AppContext;

pub fn web_popup_open(ctx: &mut AppContext, kind: i32, map: i32, stage: i32) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .web_popup_open(kind, map, stage);

    Ok(())
}
