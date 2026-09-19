use crate::Fault;

use super::AppContext;

pub fn dialog_close(ctx: &mut AppContext, dialog: u64) -> Result<(), Fault> {
    let Some(record) = ctx.dialogs.objects.get_mut(&dialog) else {
        return Err(Fault::NullPointer { site: "dialog_close" });
    };

    record.state = 2;

    let button = record.button;
    let handler = record.on_event.ok_or(Fault::BadFunctionCall { site: "dialog_close" })?;

    handler(ctx, dialog, 4, button)
}
