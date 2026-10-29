use super::AppContext;

pub fn gl_surface_ready(ctx: &AppContext) -> bool {
    !ctx.surface_lost
}
