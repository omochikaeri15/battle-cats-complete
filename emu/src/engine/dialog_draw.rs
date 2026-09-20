use crate::Fault;

use super::AppContext;

pub fn dialog_draw(ctx: &mut AppContext, dialog: u64, layer: i32) -> Result<(), Fault> {
    ctx.ui()
        .ok_or(Fault::host_missing())?
        .dialog_draw(dialog, layer);

    Ok(())
}
