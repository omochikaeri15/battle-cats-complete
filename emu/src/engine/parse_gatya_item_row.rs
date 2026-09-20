use crate::Fault;

use super::{
    AppContext, AssetStream, GatyaItem, get_column_count, obfuscate_value, read_csv_cell,
    read_csv_row,
};

pub fn parse_gatya_item_row(
    ctx: &mut AppContext,
    row: usize,
    stm: &mut AssetStream<'_>,
) -> Result<(), Fault> {
    read_csv_row(stm);

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 0) as i32).to_le_bytes());
    obfuscate_value(&mut cell);
    ctx.set_block_at(row + GatyaItem::RARITY, cell)?;
    ctx.set_block_at::<1>(
        row + GatyaItem::REFLECT_OR_STORAGE,
        [u8::from(read_csv_cell(stm, 1) != 0)],
    )?;

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 2) as i32).to_le_bytes());
    obfuscate_value(&mut cell);
    ctx.set_block_at(row + GatyaItem::PRICE, cell)?;

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 3) as i32).to_le_bytes());
    obfuscate_value(&mut cell);
    ctx.set_block_at(row + GatyaItem::STAGE_DROP_ITEM_ID, cell)?;

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 4) as i32).to_le_bytes());
    obfuscate_value(&mut cell);
    ctx.set_block_at(row + GatyaItem::QUANTITY, cell)?;
    ctx.set_i32_at(row + GatyaItem::SERVER_ID, read_csv_cell(stm, 5) as i32)?;
    ctx.set_i32_at(row + GatyaItem::CATEGORY, read_csv_cell(stm, 6) as i32)?;
    ctx.set_i32_at(row + GatyaItem::INDEX, read_csv_cell(stm, 7) as i32)?;
    ctx.set_i32_at(row + GatyaItem::SRC_ITEM_ID, read_csv_cell(stm, 8) as i32)?;
    ctx.set_i32_at(row + GatyaItem::MAIN_MENU_TYPE, read_csv_cell(stm, 9) as i32)?;
    ctx.set_i32_at(row + GatyaItem::GATYA_TICKET_ID, read_csv_cell(stm, 0xa) as i32)?;

    let mut img_id = -1i32;

    if get_column_count(stm) as i32 >= 0xa {
        img_id = read_csv_cell(stm, 0xb) as i32;
    }

    ctx.set_i32_at(row + GatyaItem::IMG_ID, img_id)
}
