use crate::Fault;

use super::{
    AppContext, AssetStream, open_asset_stream, read_cell_stream, read_csv_cell, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OfficersClubRow {
    pub club_item_id: i32,
    pub lower_limit: i32,
    pub upper_limit: i32,
    pub items: Vec<[i32; 3]>,
}

pub fn load_officers_club_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.officers_club_rows.clear();

    let Some(bytes) = open_asset_stream(ctx, b"NyankoClubData.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if read_cell_stream(stm, 0).is_empty() {
            return Ok(());
        }

        let mut items: Vec<[i32; 3]> = Vec::new();
        let key = read_csv_cell(stm, 0) as i32;
        let lower_limit = read_csv_cell(stm, 1) as i32;
        let upper_limit = read_csv_cell(stm, 2) as i32;
        let count = read_csv_cell(stm, 3) as i32;

        if count > 0 {
            let mut index = 0i32;

            while index != count {
                let category = read_csv_cell(stm, index.wrapping_mul(3).wrapping_add(4)) as i32;
                let item = read_csv_cell(stm, index.wrapping_mul(3).wrapping_add(5)) as i32;
                let value = read_csv_cell(stm, index.wrapping_mul(3).wrapping_add(6)) as i32;

                items.push([category, item, value]);
                index = index.wrapping_add(1);
            }
        }

        let record = ctx.officers_club_rows.entry(key).or_default();

        record.club_item_id = key;
        record.lower_limit = lower_limit;
        record.upper_limit = upper_limit;
        record.items = items;
    }

    Ok(())
}
