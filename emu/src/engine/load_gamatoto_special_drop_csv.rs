use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct GamatotoSpecialDrop {
    pub id: i32,
    pub stage: i32,
    pub drop_id: i32,
    pub quantity: i32,
    pub chance: i32,
    pub time_cap: i32,
}

pub fn load_gamatoto_special_drop_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"GamatotoExpedition_SpecialDropItem.csv", 0, 0)?
    else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;
        let id = read_csv_cell(stm, 0) as i32;
        let category = read_csv_cell(stm, 1) as i32;
        let cell = read_csv_cell(stm, 2) as i32;
        let mut stage = cell.wrapping_add(0x1388);

        if category == 0 {
            stage = cell;
        }

        let drop_id = read_csv_cell(stm, 3) as i32;
        let quantity = read_csv_cell(stm, 5) as i32;
        let chance = read_csv_cell(stm, 4) as i32;
        let time_cap = read_csv_cell(stm, 6) as i32;

        ctx.gamatoto_special_drops.insert(
            key,
            GamatotoSpecialDrop {
                id,
                stage,
                drop_id,
                quantity,
                chance,
                time_cap,
            },
        );
    }

    Ok(())
}
