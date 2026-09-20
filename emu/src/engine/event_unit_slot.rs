use crate::Fault;

pub fn event_unit_slot(rows: &[Vec<i32>], unit: i32) -> Result<i32, Fault> {
    let count = rows.len() as i32;

    if count <= 0 {
        return Ok(-1);
    }

    for row in rows.iter().take(count as u32 as usize) {
        if *row.get(2).ok_or(Fault::index_out_of_range(2, row.len() as i64))? == unit
        {
            return row.get(1).copied().ok_or(Fault::index_out_of_range(1, row.len() as i64));
        }
    }

    Ok(-1)
}
