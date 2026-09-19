use super::AppContext;

pub fn map_records_entry(ctx: &AppContext, map: i32) -> bool {
    ctx.entry_record_maps.contains(&map)
}
