use crate::Fault;

use super::AppContext;

pub trait Platform {
    fn item_pass_active(&mut self, item: i32) -> bool;
    fn set_keep_awake(&mut self, awake: bool);
    fn is_tablet(&mut self) -> bool;
    fn system_clock_now(&mut self) -> i64;
    fn vibrate(&mut self, delay: f64, duration: f64, amplitude: f64);
    fn cancel_vibration(&mut self);
    fn web_view_is_open(&mut self) -> bool;
    fn share_image(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn has_inquiry_code(&mut self) -> bool;
    fn connecting_show(&mut self, text: &[u8]);
    fn connecting_hide(&mut self);
    fn ranking_submit(&mut self, entry: i32, score: i32);
    fn ads_available(&mut self) -> bool;
    fn feature_enabled(&mut self, feature: i32) -> bool;
    fn config_int(&mut self, key: &[u8], min: i32, max: i32) -> i32;
    fn reward_ad_ready(&mut self, kind: i32, flag: i32) -> bool;
    fn show_rewarded_ad(&mut self, kind: i32);
    fn web_popup_open(&mut self, kind: i32, map: i32, stage: i32);
    fn labyrinth_submit(&mut self, cleared: i32, units: i32);
    fn ad_prepare(&mut self, kind: i32);
    fn event_schedule_active(&mut self, now: f64) -> Vec<Vec<i32>>;
    fn config_json_int(&mut self, section: &[u8], name: &[u8]) -> Option<i32>;
    fn notification_schedule(&mut self, flag: u8, kind: i32);
}

pub fn set_keep_awake(ctx: &mut AppContext, awake: u8) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "set_keep_awake" })?.set_keep_awake(awake != 0);

    Ok(())
}
