use super::AppContext;

pub fn map_one_time_limit(ctx: &AppContext, map: i32) -> i32 {
    ctx.map_one_time
        .range(map..)
        .next()
        .filter(|(key, _)| **key <= map)
        .map_or(0, |(_, value)| *value)
}
