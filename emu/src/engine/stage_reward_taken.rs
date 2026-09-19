use super::AppContext;

pub fn stage_reward_taken(ctx: &mut AppContext, map: i32, stage: i32) -> bool {
    if !ctx.stage_rewards_taken.contains_key(&map) {
        return false;
    }

    *ctx.stage_rewards_taken.entry(map).or_default().entry(stage).or_insert(false)
}
