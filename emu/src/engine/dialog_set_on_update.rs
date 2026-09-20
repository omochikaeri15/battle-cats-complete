use crate::Fault;

use super::{AppContext, DialogUpdateHandler};

pub fn dialog_set_on_update(
    ctx: &mut AppContext,
    dialog: u64,
    handler: Option<DialogUpdateHandler>,
) -> Result<u64, Fault> {
    ctx.dialogs
        .objects
        .get_mut(&dialog)
        .ok_or(Fault::null_pointer())?
        .on_update = handler;

    Ok(dialog)
}
