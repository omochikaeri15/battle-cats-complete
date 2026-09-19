use crate::{operation, Fault};

use super::{find_score_bonus, AppContext};

pub fn get_score_bonus(ctx: &mut AppContext, kind: i32, divisor: i32) -> Result<i32, Fault> {
    let bonus = find_score_bonus(ctx, kind)?;
    let mut result = 0;

    if let Some(values) = bonus
        && divisor != 0
        && let Some(first) = values.first()
    {
        result = operation::idiv(*first, divisor).ok_or(Fault::divide("get_score_bonus", divisor as i64))?;
    }

    Ok(result)
}
