use super::AppContext;

pub fn medal_awarded(ctx: &AppContext, medal: i32) -> bool {
    ctx.medals_awarded.contains(&medal)
}
