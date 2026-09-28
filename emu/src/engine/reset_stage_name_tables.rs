use crate::Fault;

use super::AppContext;

pub fn reset_stage_name_tables(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut cell = 0usize;

    while cell != 48 {
        ctx.set_i32_at(AppContext::DROP_MAP_STAGES + cell * 4, -1)?;
        cell += 1;
    }

    Ok(())
}
