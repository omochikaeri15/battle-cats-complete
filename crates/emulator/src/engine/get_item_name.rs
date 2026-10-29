use super::AppContext;

pub fn get_item_name(ctx: &AppContext, item: i32) -> Vec<u8> {
    if !(0..=0x112).contains(&item) {
        return Vec::new();
    }

    ctx.item_names
        .get(item as usize)
        .cloned()
        .unwrap_or_default()
}
