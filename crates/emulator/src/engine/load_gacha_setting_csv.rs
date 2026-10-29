use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_gacha_setting_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.event_gatya_settings.clear();

    let Some(bytes) = open_asset_stream(ctx, b"EventGatya_Setting.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            break;
        }

        let gatya_id = read_csv_cell(stm, 0) as i32;
        let mut column = 1i32;

        while column < get_column_count(stm) as i32 {
            let group = read_csv_cell(stm, column) as i32;
            let unit = read_csv_cell(stm, column.wrapping_add(1)) as i32;
            let value = read_csv_cell(stm, column.wrapping_add(2)) as i32;

            ctx.event_gatya_settings
                .entry(gatya_id)
                .or_default()
                .entry(group)
                .or_default()
                .insert(unit, value);

            column = column.wrapping_add(3);
        }
    }

    Ok(())
}
