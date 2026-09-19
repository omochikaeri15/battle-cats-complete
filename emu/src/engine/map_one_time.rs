use super::AppContext;

pub fn map_one_time(ctx: &AppContext, map: i32) -> bool {
    ctx.map_one_time
        .range(map..)
        .next()
        .filter(|(key, _)| **key <= map)
        .is_some_and(|(_, value)| *value != 0)
}
