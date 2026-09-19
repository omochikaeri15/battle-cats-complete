use crate::Fault;

use super::{log_analytics_event, AppContext};

pub fn point_reward_analytics(ctx: &mut AppContext, point_id: i32, reward_id: i32, reached: i32) -> Result<(), Fault> {
    log_analytics_event(ctx, 0x6a, point_id, reward_id, reached, 0)
}
