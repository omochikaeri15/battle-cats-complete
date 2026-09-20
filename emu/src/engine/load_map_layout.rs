use crate::Fault;

use super::{AppContext, load_map_layout_csv, map_layout_set_points};

pub fn load_map_layout(ctx: &mut AppContext, map_index: i32) -> Result<bool, Fault> {
    let mut record = std::mem::take(ctx.map_layouts.entry(map_index).or_default());

    load_map_layout_csv(ctx, &mut record, map_index)?;

    *ctx.map_layouts.entry(map_index).or_default() = record;

    if map_index == 0x16 {
        let mut points: Vec<u64> = Vec::new();
        let mut pushed = 0i32;

        while pushed != 0x30 {
            points.push(0);
            pushed = pushed.wrapping_add(1);
        }

        let record = ctx.map_layouts.entry(0x16).or_default();

        map_layout_set_points(record, 0, &points);
    }

    Ok(true)
}
