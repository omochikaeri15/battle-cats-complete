use crate::Fault;

use super::{
    AppContext, AssetStream, GatyaDataSet, format_localized, open_asset_stream,
    parse_gatya_data_set_row, read_cell_stream, read_csv_row,
};

pub fn load_gatya_data_set_csv(
    ctx: &mut AppContext,
    store: &mut Vec<GatyaDataSet>,
    suffix: &[u8],
) -> Result<(), Fault> {
    let name = format_localized(ctx, b"GatyaDataSet%@1.csv", suffix)?;
    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if read_cell_stream(stm, 0).is_empty() {
            return Ok(());
        }

        store.push(GatyaDataSet::default());

        if let Some(row) = store.last_mut() {
            parse_gatya_data_set_row(row, stm);
        }
    }

    Ok(())
}
