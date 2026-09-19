use super::AppContext;

pub fn dialog_top(ctx: &AppContext) -> Option<u64> {
    ctx.dialogs.active.last().copied()
}
