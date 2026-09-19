use crate::Fault;

use super::AppContext;

pub fn event_reward_set(
    ctx: &mut AppContext,
    event: i32,
    stage: i32,
    star: i32,
    kind: i32,
    value: u8,
    use_cache: i32,
) -> Result<(), Fault> {
    let slot = if kind == 2 { star } else { 0 };

    if use_cache != 0 {
        let cell = ctx
            .event_reward_cache
            .entry(event)
            .or_default()
            .entry(stage)
            .or_default();
        let byte = cell
            .get_mut(slot as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: "event_reward_set",
                index: slot as i64,
                limit: 4,
            })?;

        *byte = value;

        return Ok(());
    }

    let cell = if event != -2 {
        (event as i64) * 0x30 + (stage as i64) * 4 + slot as i64 + AppContext::EVENT_REWARDS as i64
    } else {
        (stage as i64) * 4 + slot as i64 + AppContext::EVENT_REWARDS_NEG2 as i64
    };

    ctx.set_block_at::<1>(cell as usize, [value])
}
