use super::AppContext;

pub fn aku_timer_set(ctx: &mut AppContext, key: i32, time: f64) {
    *ctx.aku_timers.entry(key).or_insert(0.0) = time;
}
