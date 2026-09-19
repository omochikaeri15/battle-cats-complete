use super::AppContext;

pub fn clear_count_rewards_has(ctx: &mut AppContext, map: i32, stage: i32) -> bool {
    if !ctx.clear_count_rewards.contains_key(&map) {
        return false;
    }

    ctx.clear_count_rewards.entry(map).or_default().contains_key(&stage)
}
