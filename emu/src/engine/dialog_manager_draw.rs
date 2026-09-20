use crate::Fault;

use super::{AppContext, dialog_draw};

pub fn dialog_manager_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    let active = ctx.dialogs.active.clone();

    for dialog in active.iter() {
        let this = ctx
            .dialogs
            .objects
            .get(dialog)
            .ok_or(Fault::null_pointer())?;

        if this.flags & 0x100 != 0 {
            continue;
        }

        let is_top = (ctx.dialogs.active.last() == Some(dialog)) as u8;

        dialog_draw(ctx, *dialog, is_top)?;
    }

    Ok(())
}
