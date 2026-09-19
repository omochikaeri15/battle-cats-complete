use super::SoundManager;

pub fn set_bgm_duck(mgr: &mut dyn SoundManager, percent: i32) {
    mgr.set_bgm_duck(percent)
}
