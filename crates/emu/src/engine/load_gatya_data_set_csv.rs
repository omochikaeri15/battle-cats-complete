use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, GatyaDataSet, format_localized, open_asset_stream,
    parse_gatya_data_set_row, parse_gatya_data_set_row_2, parse_gatya_data_set_row_3,
    parse_gatya_option_row, read_asset_stream_line, read_cell_stream, read_csv_cell,
    read_csv_row, read_tsv_row,
};

pub fn load_gatya_data_set_csv(
    ctx: &mut AppContext,
    store: &mut Vec<GatyaDataSet>,
    suffix: &[u8],
) -> Result<(), Fault> {
    let name = format_localized(ctx, b"GatyaDataSet%@1.csv", suffix)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if read_cell_stream(stm, 0).is_empty() {
                break;
            }

            store.push(GatyaDataSet::default());

            if let Some(row) = store.last_mut() {
                parse_gatya_data_set_row(row, stm);
            }
        }
    }

    let name = format_localized(ctx, b"GatyaDataSet%@2.csv", suffix)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut index = 0usize;

        while read_csv_row(stm) {
            let Some(row) = store.get_mut(index) else {
                break;
            };

            index += 1;
            parse_gatya_data_set_row_2(row, stm);
        }
    }

    let name = format_localized(ctx, b"GatyaDataSet%@3.csv", suffix)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut index = 0usize;

        while read_csv_row(stm) {
            let Some(row) = store.get_mut(index) else {
                break;
            };

            index += 1;
            parse_gatya_data_set_row_3(row, stm);
        }
    }

    let name = format_localized(ctx, b"GatyaData_Option_Set%s.tsv", suffix)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut line = Cell { at: 0, len: 0 };

        read_asset_stream_line(stm, &mut line);

        while read_tsv_row(stm) {
            let index = read_csv_cell(stm, 0) as i32;

            if index < 0 {
                continue;
            }

            let Some(row) = store.get_mut(index as u32 as usize) else {
                continue;
            };

            parse_gatya_option_row(row, stm);
        }
    }

    Ok(())
}
