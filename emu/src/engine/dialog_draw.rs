use crate::Fault;

use super::AppContext;

const SITE: &str = "dialog_draw";

pub fn dialog_draw(ctx: &mut AppContext, dialog: u64, layer: i32) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::HostMissing { site: SITE })?.dialog_draw(dialog, layer);

    Ok(())
}
