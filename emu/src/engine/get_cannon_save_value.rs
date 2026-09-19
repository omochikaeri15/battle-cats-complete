use crate::Fault;

use super::AppContext;

pub fn get_cannon_save_value(ctx: &AppContext, part_id: i32, column: i32) -> Result<i32, Fault> {
    if column == 0 {
        let Some(row) = ctx.cannon_part_rows.get(&part_id) else {
            return Ok(1);
        };

        return row
            .get(1)
            .map(|cell| cell.wrapping_add(1))
            .ok_or(Fault::IndexOutOfRange {
                site: "get_cannon_save_value",
                index: 1,
                limit: row.len() as i64,
            });
    }

    if let Some(row) = ctx.cannon_part_rows.get(&part_id) {
        let cell = column.wrapping_add(1) as i64;

        if row.len() as u64 > cell as u64 {
            return row
                .get(cell as usize)
                .copied()
                .ok_or(Fault::IndexOutOfRange {
                    site: "get_cannon_save_value",
                    index: cell,
                    limit: row.len() as i64,
                });
        }
    }

    Ok(0)
}
