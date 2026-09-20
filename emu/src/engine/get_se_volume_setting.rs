use super::SoundManager;

pub fn get_se_volume_setting(mgr: &mut dyn SoundManager) -> i32 {
    mgr.get_se_volume_setting()
}
