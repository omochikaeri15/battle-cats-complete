use std::collections::BTreeMap;

use super::obfuscate_value;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ItemShopRow {
    pub gatya_item_id: i32,
    pub item_value: [u8; 8],
    pub item_price: [u8; 8],
    pub draw_item: u8,
    pub category: Vec<u8>,
    pub imgcut: i32,
}

pub fn item_shop_entry(rows: &mut BTreeMap<i32, ItemShopRow>, shop_id: i32) -> &mut ItemShopRow {
    rows.entry(shop_id).or_insert_with(|| {
        let mut row = ItemShopRow::default();

        obfuscate_value(&mut row.item_value);
        obfuscate_value(&mut row.item_price);

        row
    })
}
