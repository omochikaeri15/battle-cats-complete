use crate::Fault;

use super::{Enigma, EnigmaActive};

pub fn enigma_active_at(enigma: &Enigma, index: i32) -> Result<EnigmaActive, Fault> {
    enigma.active.get(index as i64 as usize).copied().ok_or(Fault::OutOfRange { site: "enigma_active_at" })
}
