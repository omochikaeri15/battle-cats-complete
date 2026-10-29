use crate::Fault;

use super::{AppContext, get_stage_record};

pub fn worker_unlocked(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(get_stage_record(ctx, -2, 0, 0, 0, 0)? > 0)
}
