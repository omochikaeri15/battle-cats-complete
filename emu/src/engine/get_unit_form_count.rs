use crate::Fault;

use super::AppContext;

pub fn get_unit_form_count(ctx: &AppContext, unit_id: i32) -> Result<i32, Fault> {
    ctx.i32_at(((unit_id as i64) << 5).wrapping_add(AppContext::UNIT_FORM_COUNTS as i64) as usize)
}
