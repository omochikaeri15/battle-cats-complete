use super::AppContext;

pub fn map_xp_ad(ctx: &AppContext, map: i32) -> bool {
    ctx.xp_ad_maps.contains(&map)
}
