use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, get_equipped_orb, has_fixed_lineup};

pub fn get_orb_value_vs_trait(
    ctx: &mut AppContext,
    unit_id: i32,
    abil: i32,
    param: i32,
    traits: &BTreeMap<i32, bool>,
    dflt: i32,
) -> Result<i32, Fault> {
    let mut value = dflt;

    if has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(value);
    }

    let mut slot = 0i32;

    loop {
        let slot_count = if ctx.orb_store.slot_counts.contains_key(&unit_id) {
            *ctx.orb_store
                .slot_counts
                .get(&unit_id)
                .ok_or(Fault::KeyNotFound {
                    site: "get_orb_value_vs_trait",
                    key: unit_id as i64,
                })?
        } else {
            0
        };

        if slot >= slot_count {
            return Ok(value);
        }

        let orb_index = get_equipped_orb(ctx, unit_id, slot)?;

        if orb_index != -1 {
            let orb = ctx.orb_store.orbs.get(orb_index as i64 as usize).ok_or(
                Fault::IndexOutOfRange {
                    site: "get_orb_value_vs_trait",
                    index: orb_index as i64,
                    limit: ctx.orb_store.orbs.len() as i64,
                },
            )?;

            if orb.abil == abil {
                for (trait_bit, present) in traits {
                    if !*present {
                        continue;
                    }

                    let trait_mask = *ctx
                        .orb_store
                        .trait_masks
                        .get(orb.trait_index as i64 as usize)
                        .ok_or(Fault::IndexOutOfRange {
                            site: "get_orb_value_vs_trait",
                            index: orb.trait_index as i64,
                            limit: ctx.orb_store.trait_masks.len() as i64,
                        })?;

                    if trait_bit & trait_mask != 0 {
                        value = value.wrapping_add(*orb.values.get(param as i64 as usize).ok_or(
                            Fault::IndexOutOfRange {
                                site: "get_orb_value_vs_trait",
                                index: param as i64,
                                limit: orb.values.len() as i64,
                            },
                        )?);
                        break;
                    }
                }
            }
        }

        slot += 1;
    }
}
