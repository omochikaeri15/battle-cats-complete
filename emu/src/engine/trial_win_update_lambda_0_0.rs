use crate::Fault;

use super::{dialog_close, AppContext};

pub fn trial_win_update_lambda_0_0(ctx: &mut AppContext, dialog: u64, event: i32, _button: i32) -> Result<(), Fault> {
    match event {
        5 => {
            ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
            ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)
        }
        2 => dialog_close(ctx, dialog),
        _ => Ok(()),
    }
}
