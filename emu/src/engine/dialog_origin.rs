use crate::Fault;

use super::AppContext;

pub fn dialog_origin(ctx: &mut AppContext, dialog: u64) -> Result<(i32, i32), Fault> {
    Ok(ctx
        .ui()
        .ok_or(Fault::host_missing())?
        .dialog_origin(dialog))
}
