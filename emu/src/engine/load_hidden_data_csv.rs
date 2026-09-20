use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_cell_stream,
    read_csv_cell, read_csv_row, std_to_string,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct HiddenPairs {
    pub firsts: Vec<i32>,
    pub seconds: Vec<i32>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct HiddenLottery {
    pub first: i32,
    pub second: i32,
    pub third: i32,
    pub pairs: HiddenPairs,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct HiddenData {
    pub rates: Vec<HiddenPairs>,
    pub drops: Vec<HiddenPairs>,
    pub groups: BTreeMap<i32, BTreeMap<i32, HiddenPairs>>,
    pub lotteries: Vec<HiddenLottery>,
    pub discover_growth: Vec<i32>,
    pub times: Vec<i32>,
}

pub fn load_hidden_data_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    if let Some(bytes) = open_asset_stream(ctx, b"Hidden_rate.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if (get_column_count(stm) as i64) < 2 {
                break;
            }

            if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
                break;
            }

            ctx.hidden_data.rates.push(HiddenPairs::default());

            let mut column = 0i32;

            while column | 1 < get_column_count(stm) as i32 {
                if read_cell_stream(stm, column)
                    != std_to_string(read_csv_cell(stm, column) as i32).as_slice()
                {
                    break;
                }

                let first = read_csv_cell(stm, column) as i32;

                if let Some(row) = ctx.hidden_data.rates.last_mut() {
                    row.firsts.push(first);
                }

                let second = read_csv_cell(stm, column | 1) as i32;

                if let Some(row) = ctx.hidden_data.rates.last_mut() {
                    row.seconds.push(second);
                }

                column = column.wrapping_add(2);
            }
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Hidden_drop.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if (get_column_count(stm) as i64) < 2 {
                break;
            }

            if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
                break;
            }

            ctx.hidden_data.drops.push(HiddenPairs::default());

            let mut column = 0i32;

            while column | 1 < get_column_count(stm) as i32 {
                if read_cell_stream(stm, column)
                    != std_to_string(read_csv_cell(stm, column) as i32).as_slice()
                {
                    break;
                }

                let first = read_csv_cell(stm, column) as i32;

                if let Some(row) = ctx.hidden_data.drops.last_mut() {
                    row.firsts.push(first);
                }

                let second = read_csv_cell(stm, column | 1) as i32;

                if let Some(row) = ctx.hidden_data.drops.last_mut() {
                    row.seconds.push(second);
                }

                column = column.wrapping_add(2);
            }
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Hidden_group.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if (get_column_count(stm) as i64) < 2 {
                break;
            }

            if !cell_is_int(stm, 0) {
                break;
            }

            let group = read_csv_cell(stm, 0) as i32;
            let slot = read_csv_cell(stm, 1) as i32;

            ctx.hidden_data
                .groups
                .entry(group)
                .or_default()
                .insert(slot, HiddenPairs::default());

            let mut column = 2i32;

            while column | 1 < get_column_count(stm) as i32 {
                if !cell_is_int(stm, column) {
                    break;
                }

                let first = read_csv_cell(stm, column) as i32;

                if let Some(row) = ctx
                    .hidden_data
                    .groups
                    .entry(group)
                    .or_default()
                    .get_mut(&slot)
                {
                    row.firsts.push(first);
                }

                let second = read_csv_cell(stm, column | 1) as i32;

                if let Some(row) = ctx
                    .hidden_data
                    .groups
                    .entry(group)
                    .or_default()
                    .get_mut(&slot)
                {
                    row.seconds.push(second);
                }

                column = column.wrapping_add(2);
            }
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Hidden_lottery.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if (get_column_count(stm) as i64) < 4 {
                break;
            }

            if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
                break;
            }

            ctx.hidden_data.lotteries.push(HiddenLottery::default());

            let first = read_csv_cell(stm, 0) as i32;
            let second = read_csv_cell(stm, 1) as i32;
            let third = read_csv_cell(stm, 2) as i32;

            if let Some(row) = ctx.hidden_data.lotteries.last_mut() {
                row.first = first;
                row.second = second;
                row.third = third;
            }

            let mut column = 3i32;

            while column.wrapping_add(1) < get_column_count(stm) as i32 {
                if read_cell_stream(stm, column)
                    != std_to_string(read_csv_cell(stm, column) as i32).as_slice()
                {
                    break;
                }

                let first = read_csv_cell(stm, column) as i32;

                if let Some(row) = ctx.hidden_data.lotteries.last_mut() {
                    row.pairs.firsts.push(first);
                }

                let second = read_csv_cell(stm, column.wrapping_add(1)) as i32;

                if let Some(row) = ctx.hidden_data.lotteries.last_mut() {
                    row.pairs.seconds.push(second);
                }

                column = column.wrapping_add(2);
            }
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"Discoverlv_growth.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) && get_column_count(stm) as i64 > 0 {
            if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
                break;
            }

            let value = read_csv_cell(stm, 0) as i32;

            ctx.hidden_data.discover_growth.push(value);
        }
    }

    let Some(bytes) = open_asset_stream(ctx, b"Hidden_time.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) && get_column_count(stm) as i64 > 0 {
        if read_cell_stream(stm, 0) != std_to_string(read_csv_cell(stm, 0) as i32).as_slice() {
            break;
        }

        let value = read_csv_cell(stm, 0) as i32;

        ctx.hidden_data.times.push(value);
    }

    Ok(())
}
