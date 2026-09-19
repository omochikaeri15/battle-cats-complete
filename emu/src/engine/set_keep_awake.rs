use crate::Fault;

use super::AppContext;

pub trait Platform {
    fn set_keep_awake(&mut self, awake: bool);
    fn is_tablet(&mut self) -> bool;
    fn system_clock_now(&mut self) -> i64;
    fn vibrate(&mut self, delay: f64, duration: f64, amplitude: f64);
    fn cancel_vibration(&mut self);
}

pub fn set_keep_awake(ctx: &mut AppContext, awake: u8) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "set_keep_awake" })?.set_keep_awake(awake != 0);

    Ok(())
}
