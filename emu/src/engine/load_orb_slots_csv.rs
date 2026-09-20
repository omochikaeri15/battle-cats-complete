use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_orb_slots_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.slot_counts.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipmentslot.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            return Ok(());
        }

        if !cell_is_int(stm, 0) {
            return Ok(());
        }

        let unit_id = read_csv_cell(stm, 0) as i32;
        let count = read_csv_cell(stm, 1) as i32;

        ctx.orb_store.slot_counts.entry(unit_id).or_default().count = count;

        let mut column = 2i32;

        while column < get_column_count(stm) as i32 && cell_is_int(stm, column) {
            let row = ctx.orb_store.slot_counts.entry(unit_id).or_default();
            let value = read_csv_cell(stm, column) as i32;

            row.level_conditions.push(value);
            column += 1;
        }
    }

    Ok(())
}
