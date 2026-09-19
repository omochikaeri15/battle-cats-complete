use crate::Fault;

use super::AppContext;

pub trait SoundManager {
    fn play_audio(&mut self, sound_id: i32, volume: Option<i32>, is_bgm: bool);
    fn stop_audio(&mut self, sound_id: i32);
    fn set_bgm_duck(&mut self, percent: i32);
    fn pause_all(&mut self);
    fn set_channel(&mut self, channel: i32, value: i32);
}

pub fn sound_manager(ctx: &mut AppContext) -> Result<&mut (dyn SoundManager + 'static), Fault> {
    ctx.sound().ok_or(Fault::HostMissing {
        site: "sound_manager",
    })
}
