use super::SoundManager;

pub fn set_bgm_duck(mgr: &mut SoundManager, percent: i32) {
    mgr.bgm_duck = percent;

    if let Some(bgm) = &mgr.bgm_source {
        bgm.set_volume(percent.wrapping_mul(mgr.bgm_volume) as f32 / 10000.0f32);
    }
}
