use crate::Fault;

use super::AppContext;

pub fn get_star_level(ctx: &AppContext) -> Result<i32, Fault> {
    if ctx.i32_at(0x327efc)? != 3 && ctx.i32_at(0x327efc)? != 0x63 {
        return Ok(0);
    }

    ctx.i32_at(0x38fecc)
}
