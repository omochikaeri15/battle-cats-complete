use super::SoundManager;

pub fn sound_set_channel(mgr: &mut dyn SoundManager, channel: i32, value: i32) {
    mgr.set_channel(channel, value);
}
