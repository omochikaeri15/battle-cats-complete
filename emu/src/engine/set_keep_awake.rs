use crate::Fault;

use super::AppContext;

pub trait Platform {
    fn set_keep_awake(&mut self, awake: bool);
    fn is_tablet(&mut self) -> bool;
}

pub fn set_keep_awake(ctx: &mut AppContext, awake: u8) -> Result<(), Fault> {
    ctx.platform().ok_or(Fault::HostMissing { site: "set_keep_awake" })?.set_keep_awake(awake != 0);

    Ok(())
}
