use super::SoundManager;

pub fn get_bgm_volume_setting(mgr: &mut dyn SoundManager) -> i32 {
    mgr.get_bgm_volume_setting()
}
