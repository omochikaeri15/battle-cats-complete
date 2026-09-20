use crate::Fault;

use super::{
    AppContext, AssetStream, ZombieLotteryRow, open_asset_stream, parse_zombie_lottery_row,
    read_csv_cell, read_csv_row,
};

pub fn load_zombie_lottery_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"zombie_lottery.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;
        let mut row = ZombieLotteryRow::default();

        parse_zombie_lottery_row(&mut row, stm);
        ctx.zombie_lottery.insert(key, row);
    }

    Ok(())
}
