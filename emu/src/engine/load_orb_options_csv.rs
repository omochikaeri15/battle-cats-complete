use crate::Fault;

use super::{AppContext, AssetStream, cell_is_int, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_orb_options_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.ability_flags.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipment_option.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if !cell_is_int(stm, 0) {
            return Ok(());
        }

        let abil = read_csv_cell(stm, 0) as i32;
        let flagged = read_csv_cell(stm, 1);

        ctx.orb_store.ability_flags.entry(abil).or_insert([0, 1])[0] = u8::from(flagged != 0);

        let repeats = read_csv_cell(stm, 2);

        ctx.orb_store.ability_flags.entry(abil).or_insert([0, 1])[1] = u8::from(repeats != 0);
    }

    Ok(())
}
