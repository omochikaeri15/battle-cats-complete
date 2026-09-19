use crate::Fault;

use super::AppContext;

pub fn get_unit_form(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    ctx.i32_at(((unit_id as i64) * 4 + AppContext::UNIT_FORMS as i64) as usize)
}
