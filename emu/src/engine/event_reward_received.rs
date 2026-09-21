use crate::Fault;

use super::AppContext;

pub fn event_reward_received(
    ctx: &mut AppContext,
    event: i32,
    stage: i32,
    crown: i32,
    kind: i32,
    use_cache: i32,
) -> Result<bool, Fault> {
    let slot = if kind == 2 { crown } else { 0 };

    if use_cache != 0 {
        if !ctx.event_reward_cache.contains_key(&event) {
            return Ok(false);
        }

        if !ctx
            .event_reward_cache
            .entry(event)
            .or_default()
            .contains_key(&stage)
        {
            return Ok(false);
        }

        let cell = ctx
            .event_reward_cache
            .entry(event)
            .or_default()
            .entry(stage)
            .or_default();

        return Ok(*cell
            .get(slot as i64 as usize)
            .ok_or(Fault::index_out_of_range(slot as i64, 4))?
            != 0);
    }

    let cell = if event != -2 {
        (event as i64) * 0x30 + (stage as i64) * 4 + slot as i64 + AppContext::EVENT_REWARDS as i64
    } else {
        (stage as i64) * 4 + slot as i64 + AppContext::EVENT_REWARDS_NEG2 as i64
    };

    Ok(ctx.u8_at(cell as usize)? != 0)
}
