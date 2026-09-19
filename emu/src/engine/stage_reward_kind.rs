use crate::Fault;

use super::{AppContext, stage_reward_item};

pub fn stage_reward_kind(ctx: &AppContext, stage: i32, slot: i32) -> Result<i32, Fault> {
    if stage_reward_item(ctx, stage, slot)? >= 0x2710
        && stage_reward_item(ctx, stage, slot)? < 0x4e20
    {
        return Ok(2);
    }

    if stage_reward_item(ctx, stage, slot)? >= 0x4e20
        && stage_reward_item(ctx, stage, slot)? < 0x7530
    {
        return Ok(3);
    }

    if stage_reward_item(ctx, stage, slot)? < 0x3e8 {
        return Ok(0);
    }

    Ok((stage_reward_item(ctx, stage, slot)? < 0x2710) as i32)
}
