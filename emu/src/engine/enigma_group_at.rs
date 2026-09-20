use crate::Fault;

use super::{Enigma, EnigmaGroup};

pub fn enigma_group_at(enigma: &Enigma, index: i32) -> Result<&EnigmaGroup, Fault> {
    enigma
        .groups
        .get(index as i64 as usize)
        .ok_or(Fault::out_of_range())
}
