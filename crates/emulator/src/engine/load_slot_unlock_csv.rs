use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_slot_unlock_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"UnitSlot_UnlockData.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    let mut slot = 0usize;

    while slot != 0x13 {
        read_csv_row(stm);

        let select = read_csv_cell(stm, 0) as i32;
        let quantity = read_csv_cell(stm, 1) as i32;

        if let Some(row) = ctx.slot_unlock_rows.get_mut(slot) {
            *row = [select, quantity];
        }

        slot += 1;
    }

    Ok(())
}
