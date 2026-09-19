use crate::Fault;

use super::AppContext;

const SITE: &str = "dialog_set_back_button";

pub fn dialog_set_back_button(
    ctx: &mut AppContext,
    dialog: u64,
    button: i32,
) -> Result<u64, Fault> {
    ctx.dialogs
        .objects
        .get_mut(&dialog)
        .ok_or(Fault::NullPointer { site: SITE })?
        .back_button = button;

    Ok(dialog)
}
