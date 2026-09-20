use crate::Fault;

use super::{AppContext, get_equipped_orb, has_fixed_lineup};

pub fn get_orb_value_sum(
    ctx: &mut AppContext,
    unit_id: i32,
    abil: i32,
    param: i32,
    dflt: i32,
) -> Result<i32, Fault> {
    let mut value = dflt;

    if has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(value);
    }

    let mut slot = 0i32;

    loop {
        let slot_count = if ctx.orb_store.slot_counts.contains_key(&unit_id) {
            ctx.orb_store
                .slot_counts
                .get(&unit_id)
                .map(|row| row.count)
                .ok_or(Fault::key_not_found(unit_id as i64))?
        } else {
            0
        };

        if slot >= slot_count {
            return Ok(value);
        }

        let orb_index = get_equipped_orb(ctx, unit_id, slot)?;

        if orb_index != -1 {
            let orb = ctx.orb_store.orbs.get(orb_index as i64 as usize).ok_or(
                Fault::index_out_of_range(orb_index as i64, ctx.orb_store.orbs.len() as i64),
            )?;

            if orb.abil == abil {
                value = value.wrapping_add(*orb.values.get(param as i64 as usize).ok_or(
                    Fault::index_out_of_range(param as i64, orb.values.len() as i64),
                )?);
            }
        }

        slot += 1;
    }
}
