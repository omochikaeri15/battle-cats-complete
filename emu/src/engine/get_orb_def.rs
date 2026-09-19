use crate::Fault;

use super::{OrbDef, OrbStore};

pub fn get_orb_def(store: &OrbStore, orb: i32) -> Result<&OrbDef, Fault> {
    store.orbs.get(orb as i64 as usize).ok_or(Fault::IndexOutOfRange { site: "get_orb_def", index: orb as i64, limit: store.orbs.len() as i64 })
}
