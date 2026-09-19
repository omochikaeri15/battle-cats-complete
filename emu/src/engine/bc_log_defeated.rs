use crate::Fault;

use super::{AppContext, bc_log};

pub fn bc_log_defeated(ctx: &mut AppContext, skip: i32) -> Result<(), Fault> {
    if skip != 0 {
        return Ok(());
    }

    bc_log(ctx, b"BcLogDefeated")
}
