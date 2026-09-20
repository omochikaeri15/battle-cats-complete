use crate::Fault;

use super::{AppContext, OrbStore, get_equipped_orb};

pub fn has_orb(ctx: &AppContext, store: &OrbStore, unit_id: i32, abil: i32) -> Result<bool, Fault> {
    let mut slot = 0i32;

    loop {
        let slot_count = if store.slot_counts.contains_key(&unit_id) {
            store.slot_counts.get(&unit_id).map(|row| row.count).ok_or(Fault::KeyNotFound {
                site: "has_orb",
                key: unit_id as i64,
            })?
        } else {
            0
        };

        if slot >= slot_count {
            return Ok(slot < slot_count);
        }

        let orb_index = get_equipped_orb(ctx, unit_id, slot)?;

        if orb_index != -1 {
            let orb = store
                .orbs
                .get(orb_index as i64 as usize)
                .ok_or(Fault::IndexOutOfRange {
                    site: "has_orb",
                    index: orb_index as i64,
                    limit: store.orbs.len() as i64,
                })?;

            if orb.abil == abil {
                return Ok(slot < slot_count);
            }
        }

        slot += 1;
    }
}
