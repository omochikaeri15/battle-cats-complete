use super::{clear_count_rewards_has, AppContext};

pub fn clear_count_rewards_get(ctx: &mut AppContext, map: i32, stage: i32) -> Vec<[i32; 2]> {
    if !clear_count_rewards_has(ctx, map, stage) {
        return vec![[-1, 0]];
    }

    ctx.clear_count_rewards.entry(map).or_default().entry(stage).or_default().clone()
}
