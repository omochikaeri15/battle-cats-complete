use crate::Fault;

use super::{AppContext, DialogUpdateHandler};

const SITE: &str = "dialog_set_on_update";

pub fn dialog_set_on_update(ctx: &mut AppContext, dialog: u64, handler: Option<DialogUpdateHandler>) -> Result<u64, Fault> {
    ctx.dialogs.objects.get_mut(&dialog).ok_or(Fault::NullPointer { site: SITE })?.on_update = handler;

    Ok(dialog)
}
