use crate::engine::SoundManager;

pub struct SilentSound;

impl SoundManager for SilentSound {
    fn play_audio(&mut self, _sound_id: i32, _volume: Option<i32>, _is_bgm: bool) {}

    fn stop_audio(&mut self, _sound_id: i32) {}

    fn set_bgm_duck(&mut self, _percent: i32) {}

    fn pause_all(&mut self) {}

    fn set_channel(&mut self, _channel: i32, _value: i32) {}

    fn get_bgm_volume_setting(&mut self) -> i32 {
        0
    }

    fn get_se_volume_setting(&mut self) -> i32 {
        0
    }

    fn set_bgm_volume_setting(&mut self, _percent: i32) {}

    fn set_se_volume_setting(&mut self, _percent: i32) {}
}
