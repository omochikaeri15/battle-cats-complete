use super::AppContext;

pub fn map_guerrilla_set(ctx: &AppContext, map: i32) -> i32 {
    ctx.map_guerrilla_sets
        .range(map..)
        .next()
        .filter(|(key, _)| **key <= map)
        .map_or(0, |(_, value)| *value)
}
