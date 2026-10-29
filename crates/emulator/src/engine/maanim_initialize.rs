use crate::Fault;

use super::{Mamodel, maanim_animate};

pub fn maanim_initialize(model: &mut Mamodel, flags: u8) -> Result<(), Fault> {
    maanim_animate(model, None, 0, 0, 1, flags)
}
