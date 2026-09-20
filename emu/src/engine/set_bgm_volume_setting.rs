use super::SoundManager;

pub fn set_bgm_volume_setting(mgr: &mut dyn SoundManager, percent: i32) {
    mgr.set_bgm_volume_setting(percent)
}
