use crate::Fault;

use super::{bc_log, AppContext};

pub fn bc_log_defeated(ctx: &mut AppContext, skip: i32) -> Result<(), Fault> {
    if skip != 0 {
        return Ok(());
    }

    bc_log(ctx, b"BcLogDefeated")
}
