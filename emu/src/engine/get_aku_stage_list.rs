use super::AppContext;

pub fn get_aku_stage_list(ctx: &mut AppContext, key: i32) -> Vec<i32> {
    ctx.aku_stage_lists.entry(key).or_default().to_vec()
}
