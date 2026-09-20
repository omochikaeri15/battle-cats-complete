use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream,
    read_csv_cell, read_csv_row, string_format_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapLayout {
    pub color: [i32; 3],
    pub flags_a: Vec<bool>,
    pub flags_b: Vec<bool>,
    pub points: Vec<Vec<u64>>,
}

pub fn load_map_layout_csv(
    ctx: &mut AppContext,
    record: &mut MapLayout,
    map_index: i32,
) -> Result<(), Fault> {
    let name = string_format_int(ctx, b"MapData_%03d.csv", map_index)?;
    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    record.points.clear();

    read_csv_row(stm);

    record.color[0] = read_csv_cell(stm, 0) as i32;
    record.color[1] = read_csv_cell(stm, 1) as i32;
    record.color[2] = read_csv_cell(stm, 2) as i32;

    let default_a = read_csv_cell(stm, 3) as i32;
    let mut default_b = true;

    if cell_is_int(stm, 4) {
        default_b = read_csv_cell(stm, 4) != 0;
    }

    while read_csv_row(stm) && get_column_count(stm) as i64 > 0 && read_csv_cell(stm, 0) != 0 {
        let count = read_csv_cell(stm, 0) as i32;
        let mut points: Vec<u64> = Vec::new();

        if cell_is_int(stm, 1) {
            let value = read_csv_cell(stm, 1) as i32;

            record.flags_a.push(value != 0);
        } else {
            record.flags_a.push(default_a != 0);
        }

        if cell_is_int(stm, 2) {
            let value = read_csv_cell(stm, 2) as i32;

            record.flags_b.push(value != 0);
        } else {
            record.flags_b.push(default_b);
        }

        if count > 0 {
            let mut index = 0i32;

            while index != count {
                read_csv_row(stm);

                let x = read_csv_cell(stm, 0) as i32;
                let y = read_csv_cell(stm, 1) as i32;

                points.push(u64::from(x as u32) | u64::from(y as u32) << 0x20);
                index = index.wrapping_add(1);
            }
        }

        record.points.push(points);
    }

    Ok(())
}
