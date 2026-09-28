use crate::Fault;

use super::{AppContext, DialogDrawHandler};

pub fn dialog_set_on_draw(
    ctx: &mut AppContext,
    dialog: u64,
    on_draw: Option<DialogDrawHandler>,
) -> Result<u64, Fault> {
    ctx.dialogs
        .objects
        .get_mut(&dialog)
        .ok_or(Fault::null_pointer())?
        .on_draw = on_draw;

    Ok(dialog)
}
