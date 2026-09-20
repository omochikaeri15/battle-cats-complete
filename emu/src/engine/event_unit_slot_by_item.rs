use crate::Fault;

pub fn event_unit_slot_by_item(rows: &[Vec<i32>], item: i32) -> Result<i32, Fault> {
    let count = rows.len() as i32;

    if count <= 0 {
        return Ok(-1);
    }

    for row in rows.iter().take(count as u32 as usize) {
        if *row.first().ok_or(Fault::index_out_of_range(0, 0))? == item
        {
            return row.get(1).copied().ok_or(Fault::index_out_of_range(1, row.len() as i64));
        }
    }

    Ok(-1)
}
