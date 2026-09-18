use super::SoundManager;

pub fn play_sound(mgr: &mut dyn SoundManager, sound_id: i32, volume: Option<i32>) {
    mgr.play_audio(sound_id, volume, false)
}
