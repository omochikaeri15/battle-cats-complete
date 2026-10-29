use super::BaseShake;

pub fn base_shake_reset(shake: &mut BaseShake) {
    shake.id = -1;
    shake.frame = 0;
    shake.unknown_2 = 0;
    shake.offset = 0;
}
