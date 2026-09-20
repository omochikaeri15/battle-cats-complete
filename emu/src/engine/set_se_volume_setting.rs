use super::SoundManager;

pub fn set_se_volume_setting(mgr: &mut dyn SoundManager, percent: i32) {
    mgr.set_se_volume_setting(percent)
}
