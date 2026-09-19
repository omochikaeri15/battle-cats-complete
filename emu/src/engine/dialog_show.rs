use crate::Fault;

use super::{AppContext, DialogEventHandler, dialog_new};

pub fn dialog_show(
    ctx: &mut AppContext,
    text: &[u8],
    first_button: i32,
    second_button: i32,
    flags: i32,
    on_event: Option<DialogEventHandler>,
) -> Result<u64, Fault> {
    let dialog = dialog_new(ctx, 1, text, first_button, second_button, flags, on_event)?;

    if flags & 0x20 != 0 && !ctx.dialogs.active.is_empty() {
        ctx.dialogs.queued.push(dialog);

        return Ok(dialog);
    }

    ctx.dialogs.active.push(dialog);

    if flags & 0x40 == 0 {
        ctx.set_block_at::<1>(AppContext::BACK_PRESSED, [0])?;
        ctx.set_block_at::<1>(AppContext::TOUCH_BEGAN, [0])?;
        ctx.set_block_at::<1>(AppContext::TOUCH_RELEASED, [0])?;
        ctx.set_block_at::<1>(AppContext::TOUCH_IS_DOWN, [0])?;
    }

    Ok(dialog)
}
