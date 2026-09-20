use crate::Fault;

use super::{OrbDef, OrbStore};

pub fn get_orb_def(store: &OrbStore, orb: i32) -> Result<&OrbDef, Fault> {
    store
        .orbs
        .get(orb as i64 as usize)
        .ok_or(Fault::index_out_of_range(orb as i64, store.orbs.len() as i64))
}
