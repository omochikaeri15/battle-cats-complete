use super::AppContext;

pub fn is_conditioned_map(ctx: &AppContext, map_id: i32) -> bool {
    ctx.conditioned_maps.contains(&map_id)
}
