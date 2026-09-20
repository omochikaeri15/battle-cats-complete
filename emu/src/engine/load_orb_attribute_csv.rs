use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_orb_attribute_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.trait_masks.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipment_attribute.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let mut mask = 0i32;
        let mut column = 0i32;

        while column < get_column_count(stm) as i32 && cell_is_int(stm, column) {
            let bit = if read_csv_cell(stm, column) == 1 {
                1i32.wrapping_shl(column as u32)
            } else {
                0
            };

            mask |= bit;
            column += 1;
        }

        ctx.orb_store.trait_masks.push(mask);
    }

    Ok(())
}
