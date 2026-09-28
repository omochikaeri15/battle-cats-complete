use super::StageNameTable;

pub fn stage_name_table_size(table: &mut StageNameTable, crowns: i32, stages: i32) {
    let count = ((table.names.len() * 3) as u32).wrapping_mul(0xaaaa_aaab) as i32;
    let maps = count as i64 as usize;

    table.stage_unlock.resize(maps, Vec::new());
    table.stages_cleared.resize(maps, Vec::new());
    table.stage_record.resize(maps, Vec::new());
    table.map_open.resize(maps, Vec::new());
    table.map_flag.resize(maps, 0);
    table.map_opened.resize(maps, false);
    table.map_stamp.resize(maps, 0);
    table.map_stamp_alt.resize(maps, 0);

    let mut map = 0i64;

    while map < count as i64 {
        if let Some(row) = table.stage_unlock.get_mut(map as usize) {
            row.resize(crowns as i64 as usize, 0);
        }

        if let Some(row) = table.stages_cleared.get_mut(map as usize) {
            row.resize(crowns as i64 as usize, 0);
        }

        if let Some(row) = table.stage_record.get_mut(map as usize) {
            row.resize(crowns as i64 as usize, Vec::new());
        }

        if let Some(row) = table.map_open.get_mut(map as usize) {
            row.resize(crowns as i64 as usize, 0);
        }

        if crowns > 0 {
            let mut crown = 0i64;

            while crown != crowns as i64 {
                if let Some(cells) = table
                    .stage_record
                    .get_mut(map as usize)
                    .and_then(|row| row.get_mut(crown as usize))
                {
                    cells.resize(stages as i64 as usize, 0);
                }

                crown += 1;
            }
        }

        map += 1;
    }
}
