use super::AppContext;

pub fn map_xp_ad(ctx: &AppContext, map: i32) -> bool {
    ctx.map_options.double_xp_ad_maps.contains(&map)
}
