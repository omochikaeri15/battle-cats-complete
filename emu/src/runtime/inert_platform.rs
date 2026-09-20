use std::time::{SystemTime, UNIX_EPOCH};

use std::cell::Cell;
use std::rc::Rc;

use crate::engine::Platform;

#[derive(Default)]
pub struct DeviceProfile {
    pub tablet: Cell<bool>,
    pub side_inset: Cell<i32>,
}

#[derive(Default)]
pub struct InertPlatform {
    pub profile: Rc<DeviceProfile>,
}

impl Platform for InertPlatform {
    fn set_keep_awake(&mut self, _awake: bool) {}

    fn item_pass_active(&mut self, _item: i32) -> bool {
        false
    }

    fn is_tablet(&mut self) -> bool {
        self.profile.tablet.get()
    }

    fn screen_window_ratio(&mut self) -> f32 {
        1.0
    }

    fn safe_inset_left(&mut self) -> i32 {
        self.profile.side_inset.get()
    }

    fn safe_inset_top(&mut self) -> i32 {
        0
    }

    fn safe_inset_right(&mut self) -> i32 {
        self.profile.side_inset.get()
    }

    fn safe_inset_bottom(&mut self) -> i32 {
        0
    }

    fn system_clock_now(&mut self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_micros() as i64)
    }

    fn vibrate(&mut self, _gate: f64, _duration: f64, _strength: f64) {}

    fn cancel_vibration(&mut self) {}

    fn has_vibrator(&mut self) -> bool {
        true
    }

    fn web_view_is_open(&mut self) -> bool {
        false
    }

    fn share_image(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    fn has_inquiry_code(&mut self) -> bool {
        false
    }

    fn connecting_show(&mut self, _text: &[u8]) {}

    fn connecting_hide(&mut self) {}

    fn ranking_submit(&mut self, _entry: i32, _score: i32) {}

    fn ads_available(&mut self) -> bool {
        false
    }

    fn feature_enabled(&mut self, _feature: i32) -> bool {
        false
    }

    fn config_int(&mut self, _key: &[u8], min: i32, _max: i32) -> i32 {
        min
    }

    fn reward_ad_ready(&mut self, _kind: i32, _flag: i32) -> bool {
        false
    }

    fn show_rewarded_ad(&mut self, _kind: i32) {}

    fn web_popup_open(&mut self, _kind: i32, _map: i32, _stage: i32) {}

    fn labyrinth_submit(&mut self, _cleared: i32, _units: i32) {}

    fn ad_prepare(&mut self, _kind: i32) {}

    fn event_schedule_active(&mut self, _now: f64) -> Vec<Vec<i32>> {
        Vec::new()
    }

    fn config_json_int(&mut self, _section: &[u8], _name: &[u8]) -> Option<i32> {
        None
    }

    fn notification_schedule(&mut self, _flag: u8, _kind: i32) {}
}
