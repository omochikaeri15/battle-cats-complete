use super::AppContext;

pub fn get_item_description(ctx: &AppContext, item: i32) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();

    if (0..=0x112).contains(&item) {
        let record = ctx.item_descriptions.get(item as usize).cloned().unwrap_or_default();

        lines.push(record[0].clone());
        lines.push(record[1].clone());
        lines.push(record[2].clone());
    }

    if lines.is_empty() {
        lines.push(Vec::new());
        lines.push(Vec::new());
        lines.push(Vec::new());
    }

    lines
}
