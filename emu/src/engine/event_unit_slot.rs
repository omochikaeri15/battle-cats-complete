use crate::Fault;

pub fn event_unit_slot(rows: &[Vec<i32>], unit: i32) -> Result<i32, Fault> {
    let count = rows.len() as i32;

    if count <= 0 {
        return Ok(-1);
    }

    for row in rows.iter().take(count as u32 as usize) {
        if *row.get(2).ok_or(Fault::IndexOutOfRange {
            site: "event_unit_slot",
            index: 2,
            limit: row.len() as i64,
        })? == unit
        {
            return row.get(1).copied().ok_or(Fault::IndexOutOfRange {
                site: "event_unit_slot",
                index: 1,
                limit: row.len() as i64,
            });
        }
    }

    Ok(-1)
}
