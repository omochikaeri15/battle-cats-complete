use super::{AppContext, DialogDrawHandler};

pub fn dialog_set_on_draw(ctx: &mut AppContext, dialog: u64, on_draw: Option<DialogDrawHandler>) -> u64 {
    if let Some(record) = ctx.dialogs.objects.get_mut(&dialog) {
        record.on_draw = on_draw;
    }

    dialog
}
