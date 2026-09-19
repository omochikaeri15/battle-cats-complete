use std::time::{SystemTime, UNIX_EPOCH};

use crate::engine::Platform;

pub struct InertPlatform;

impl Platform for InertPlatform {
    fn set_keep_awake(&mut self, _awake: bool) {}

    fn is_tablet(&mut self) -> bool {
        false
    }

    fn system_clock_now(&mut self) -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_micros() as i64)
    }

    fn vibrate(&mut self, _gate: f64, _duration: f64, _strength: f64) {}

    fn cancel_vibration(&mut self) {}
}
