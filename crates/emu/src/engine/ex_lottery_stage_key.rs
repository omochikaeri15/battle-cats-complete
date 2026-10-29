use crate::Fault;

use super::AppContext;

pub fn ex_lottery_stage_key(ctx: &AppContext, pick: i32) -> Result<i32, Fault> {
    let entry = ctx
        .ex_lottery
        .get(pick as i64 as usize)
        .ok_or(Fault::out_of_range())?;

    Ok(entry[0].wrapping_mul(100).wrapping_add(entry[1]))
}
