use crate::Fault;

use super::AppContext;

pub fn get_cannon_save_level(ctx: &AppContext, part_id: i32) -> Result<i32, Fault> {
    let Some(row) = ctx.cannon_part_rows.get(&part_id) else {
        return Ok(0);
    };

    row.get(1).copied().ok_or(Fault::index_out_of_range(1, row.len() as i64))
}
