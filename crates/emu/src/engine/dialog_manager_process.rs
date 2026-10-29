use crate::Fault;

use super::{AppContext, dialog_update};

pub fn dialog_manager_process(ctx: &mut AppContext) -> Result<(), Fault> {
    let snapshot = ctx.dialogs.active.clone();
    let mut finished: Option<u64> = None;

    for dialog in snapshot.iter() {
        let is_top = (snapshot.last() == Some(dialog)) as u8;

        dialog_update(ctx, *dialog, is_top)?;

        if !ctx.dialogs.active.contains(dialog) {
            return Ok(());
        }

        let this = ctx
            .dialogs
            .objects
            .get(dialog)
            .ok_or(Fault::null_pointer())?;

        if this.state == 3 {
            finished = Some(*dialog);
        }
    }

    if let Some(top) = ctx.dialogs.active.last() {
        let this = ctx.dialogs.objects.get(top).ok_or(Fault::null_pointer())?;

        if this.flags & 0x40 == 0 {
            ctx.set_block_at::<1>(AppContext::BACK_PRESSED, [0])?;
            ctx.set_block_at::<1>(AppContext::TOUCH_BEGAN, [0])?;
            ctx.set_block_at::<1>(AppContext::TOUCH_RELEASED, [0])?;
            ctx.set_block_at::<1>(AppContext::TOUCH_IS_DOWN, [0])?;
        }
    }

    let Some(finished) = finished else {
        return Ok(());
    };

    ctx.dialogs.active.retain(|held| *held != finished);
    ctx.dialogs.objects.remove(&finished);

    if !ctx.dialogs.queued.is_empty() && ctx.dialogs.active.is_empty() {
        let next = ctx.dialogs.queued.remove(0);

        ctx.dialogs.active.push(next);
    }

    Ok(())
}
