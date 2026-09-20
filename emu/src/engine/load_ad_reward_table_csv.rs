use crate::Fault;

use super::{AppContext, AssetStream, get_column_count, open_asset_stream, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct AdRewardRow {
    pub big_bag: i32,
    pub user_rank: i32,
    pub item_id: i32,
    pub pairs: Vec<[i32; 2]>,
}

pub fn load_ad_reward_table_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"Adreward_table.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let big_bag = read_csv_cell(stm, 0) as i32;
        let user_rank = read_csv_cell(stm, 1) as i32;
        let item_id = read_csv_cell(stm, 2) as i32;
        let mut pairs: Vec<[i32; 2]> = Vec::new();
        let mut column = 3i32;

        while column < (get_column_count(stm) as i32).wrapping_sub(1) {
            let first = read_csv_cell(stm, column) as i32;
            let second = read_csv_cell(stm, column.wrapping_add(1)) as i32;

            pairs.push([first, second]);
            column = column.wrapping_add(2);
        }

        ctx.ad_reward_rows.push(AdRewardRow {
            big_bag,
            user_rank,
            item_id,
            pairs,
        });
    }

    ctx.ad_reward_rows.sort_by_key(|row| row.item_id);

    Ok(())
}
