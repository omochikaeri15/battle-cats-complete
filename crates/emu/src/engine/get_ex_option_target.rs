use super::AppContext;

pub fn get_ex_option_target(ctx: &mut AppContext, map_id: i32) -> i32 {
    if !ctx.ex_option_targets.contains_key(&map_id) {
        return -1;
    }

    *ctx.ex_option_targets.entry(map_id).or_insert(0)
}
