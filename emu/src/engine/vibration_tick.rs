use crate::Fault;

use super::{now_seconds, vibrate, AppContext};

pub fn vibration_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.battle_event_latch.kind == 0 {
        return Ok(());
    }

    if ctx.u8_at(AppContext::VIBRATION_ENABLED)? == 0 {
        return Ok(());
    }

    ctx.battle_event_latch.time = now_seconds(ctx)?;

    let latch = &mut ctx.battle_event_latch;
    let (kind, faction) = (latch.kind, latch.faction);

    latch.strength = latch.gates.entry(kind).or_default().entry(faction).or_default()[1];

    let gate = latch.gates.entry(kind).or_default().entry(faction).or_default()[0];
    let strength = latch.strength;
    let tail = latch.gates.entry(kind).or_default().entry(faction).or_default()[2];

    vibrate(ctx, gate, strength, tail)?;

    ctx.battle_event_latch.kind = 0;

    Ok(())
}
