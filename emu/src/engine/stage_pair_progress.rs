use super::AppContext;

pub fn stage_pair_progress(ctx: &AppContext, map: i32) -> i32 {
    ctx.stage_pair_progress.get(&map).map_or(0, |progress| progress[1])
}
