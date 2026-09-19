use super::{AppContext, pause_vibration};

pub fn vibration_clear(ctx: &mut AppContext) {
    pause_vibration();

    ctx.battle_event_latch.kind = 0;
    ctx.battle_event_latch.time = 0.0;
}
