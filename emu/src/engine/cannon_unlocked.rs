use crate::Fault;

use super::{get_stage_record, AppContext};

pub fn cannon_unlocked(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(get_stage_record(ctx, -2, 0, 1, 0, 0)? > 0)
}
