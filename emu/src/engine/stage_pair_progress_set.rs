use super::AppContext;

pub fn stage_pair_progress_set(ctx: &mut AppContext, map: i32, stage: i32, value: i32) {
    if value != 0 {
        ctx.stage_pair_progress.entry(map).or_default()[0] = stage;
        ctx.stage_pair_progress.entry(map).or_default()[1] = value;

        return;
    }

    ctx.stage_pair_progress.remove(&map);
}
