use crate::engine::SoundManager;

pub struct SilentSound;

impl SoundManager for SilentSound {
    fn play_audio(&mut self, _sound_id: i32, _volume: Option<i32>, _is_bgm: bool) {}
}
