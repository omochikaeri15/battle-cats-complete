use crate::Fault;

use super::{
    AppContext, AssetStream, UnlockPopupRow, open_asset_stream, parse_unlock_popup_row,
    read_csv_cell, read_csv_row, read_tsv_row,
};

pub fn load_unlock_popup_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"unlockPopup.tsv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_tsv_row(stm) {
        let mut row = UnlockPopupRow::default();

        parse_unlock_popup_row(&mut row, stm);

        let key = read_csv_cell(stm, 0) as i32;

        ctx.unlock_popup_rows.insert(key, row);
    }

    Ok(())
}
