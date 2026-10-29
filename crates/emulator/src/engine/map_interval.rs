use super::AppContext;

pub fn map_interval(ctx: &AppContext, map: i32) -> i32 {
    ctx.map_options.interval
        .range(map..)
        .next()
        .filter(|(key, _)| **key <= map)
        .map_or(0, |(_, value)| *value)
}
