use crate::Fault;

use super::OrbStore;

pub fn get_orb_slot_count(store: &OrbStore, unit_id: i32) -> Result<i32, Fault> {
    if !store.slot_counts.contains_key(&unit_id) {
        return Ok(0);
    }

    store.slot_counts.get(&unit_id).copied().ok_or(Fault::KeyNotFound { site: "get_orb_slot_count", key: unit_id as i64 })
}
