use crate::fault::Fault;

use super::AppContext;

pub trait SoundManager {
    fn play_audio(&mut self, sound_id: i32, volume: Option<i32>, is_bgm: bool);
}

pub fn sound_manager(ctx: &mut AppContext) -> Result<&mut (dyn SoundManager + 'static), Fault> {
    ctx.sound().ok_or(Fault::HostMissing { site: "sound_manager" })
}
