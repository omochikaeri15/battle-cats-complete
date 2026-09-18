use crate::Fault;

use super::{read_csv_cell, read_csv_row, AppContext, AssetStream};

const BUY_TABLE: usize = 0x4af08;
const BUY_STRIDE: usize = 0x100;
const BUY_ROWS: usize = 0x36c;
const BUY_COLUMNS: i32 = 0x3f;

const GROWTH_STRIDE: usize = 0x50;
const GROWTH_SPAN: usize = 0x111c0;
const GROWTH_COLUMNS: i32 = 0x14;

pub fn load_cat_data_files(
    ctx: &mut AppContext,
    unitbuy: &mut AssetStream<'_>,
    unitlevel: &mut AssetStream<'_>,
    unitexp: &mut AssetStream<'_>,
) -> Result<(), Fault> {
    let mut buy = BUY_TABLE;

    for row in 0..BUY_ROWS {
        read_csv_row(unitbuy);

        let mut column = 0;

        while column != BUY_COLUMNS {
            let key = ctx.i32_at((row << 8) + 0x4b004)?;
            ctx.set_i32_at(buy + (column as usize) * 4, read_csv_cell(unitbuy, column) as i32 ^ key)?;
            column += 1;
        }

        buy += BUY_STRIDE;
    }

    let mut growth_at = 0;

    while growth_at != GROWTH_SPAN {
        read_csv_row(unitlevel);

        let mut column = 0;

        while column != GROWTH_COLUMNS {
            ctx.set_i32_at(growth_at + AppContext::UNIT_LEVEL_CURVE + (column as usize) * 4, read_csv_cell(unitlevel, column) as i32)?;
            column += 1;
        }

        growth_at += GROWTH_STRIDE;
    }

    let mut growth_at = 0;

    while growth_at != GROWTH_SPAN {
        read_csv_row(unitexp);

        let mut column = 0;

        while column != GROWTH_COLUMNS {
            ctx.set_i32_at(growth_at + AppContext::UNIT_EXP_CURVE + (column as usize) * 4, read_csv_cell(unitexp, column) as i32)?;
            column += 1;
        }

        growth_at += GROWTH_STRIDE;
    }

    Ok(())
}
