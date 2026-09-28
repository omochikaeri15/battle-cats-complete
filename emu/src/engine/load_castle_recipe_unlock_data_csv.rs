use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CastleRecipeUnlockRow {
    pub recipe: i32,
    pub dev_level: i32,
    pub stage: i32,
    pub user_rank: i32,
    pub attack_level: i32,
    pub charge_level: i32,
}

pub fn load_castle_recipe_unlock_data_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"CastleRecipeUnlockData.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;

        *ctx.castle_recipe_unlock_data.entry(key).or_default() = CastleRecipeUnlockRow::default();
        ctx.castle_recipe_unlock_data.entry(key).or_default().recipe = read_csv_cell(stm, 1) as i32;
        ctx.castle_recipe_unlock_data.entry(key).or_default().dev_level = read_csv_cell(stm, 2) as i32;
        ctx.castle_recipe_unlock_data.entry(key).or_default().stage = read_csv_cell(stm, 3) as i32;
        ctx.castle_recipe_unlock_data.entry(key).or_default().user_rank = read_csv_cell(stm, 4) as i32;
        ctx.castle_recipe_unlock_data.entry(key).or_default().attack_level = read_csv_cell(stm, 5) as i32;
        ctx.castle_recipe_unlock_data.entry(key).or_default().charge_level = read_csv_cell(stm, 6) as i32;
    }

    Ok(())
}
