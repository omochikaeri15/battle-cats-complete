use super::{AssetStream, get_column_count, obfuscate_value, read_csv_cell, read_csv_row};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct GatyaItemRow {
    pub rarity: [u8; 8],
    pub price: [u8; 8],
    pub stage_drop_item_id: [u8; 8],
    pub quantity: [u8; 8],
    pub server_id: i32,
    pub category: i32,
    pub index: i32,
    pub src_item_id: i32,
    pub main_menu_type: i32,
    pub gatya_ticket_id: i32,
    pub img_id: i32,
    pub reflect_or_storage: u8,
}

pub fn parse_gatya_item_row(row: &mut GatyaItemRow, stm: &mut AssetStream<'_>) {
    read_csv_row(stm);

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 0) as i32).to_le_bytes());
    obfuscate_value(&mut cell);

    row.rarity = cell;
    row.reflect_or_storage = u8::from(read_csv_cell(stm, 1) != 0);

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 2) as i32).to_le_bytes());
    obfuscate_value(&mut cell);

    row.price = cell;

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 3) as i32).to_le_bytes());
    obfuscate_value(&mut cell);

    row.stage_drop_item_id = cell;

    let mut cell = [0u8; 8];

    cell[..4].copy_from_slice(&(read_csv_cell(stm, 4) as i32).to_le_bytes());
    obfuscate_value(&mut cell);

    row.quantity = cell;
    row.server_id = read_csv_cell(stm, 5) as i32;
    row.category = read_csv_cell(stm, 6) as i32;
    row.index = read_csv_cell(stm, 7) as i32;
    row.src_item_id = read_csv_cell(stm, 8) as i32;
    row.main_menu_type = read_csv_cell(stm, 9) as i32;
    row.gatya_ticket_id = read_csv_cell(stm, 0xa) as i32;

    let mut img_id = -1i32;

    if get_column_count(stm) as i32 >= 0xa {
        img_id = read_csv_cell(stm, 0xb) as i32;
    }

    row.img_id = img_id;
}
