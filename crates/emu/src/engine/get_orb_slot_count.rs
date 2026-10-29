use crate::Fault;

use super::OrbStore;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbSlotRow {
    pub count: i32,
    pub level_conditions: Vec<i32>,
}

pub fn get_orb_slot_count(store: &OrbStore, unit_id: i32) -> Result<i32, Fault> {
    if !store.slot_counts.contains_key(&unit_id) {
        return Ok(0);
    }

    store
        .slot_counts
        .get(&unit_id)
        .map(|row| row.count)
        .ok_or(Fault::key_not_found(unit_id as i64))
}
