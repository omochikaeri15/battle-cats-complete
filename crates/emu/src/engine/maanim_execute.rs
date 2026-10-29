use crate::Fault;

use super::{Maanim, Mamodel, maanim_animate};

pub fn maanim_execute(
    model: &mut Mamodel,
    anim: Option<&Maanim>,
    frame: i32,
    flags: u8,
) -> Result<(), Fault> {
    maanim_animate(model, anim, frame, 0, 1, flags)
}
