use std::collections::BTreeMap;

use crate::Fault;

use super::{get_equipped_orb, has_fixed_lineup, AppContext};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbDef {
    pub grade: i32,
    pub trait_index: i32,
    pub abil: i32,
    pub values: Vec<i32>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbStore {
    pub orbs: Vec<OrbDef>,
    pub trait_masks: Vec<i32>,
    pub slot_counts: BTreeMap<i32, i32>,
}

pub fn get_orb_value_max(ctx: &mut AppContext, unit_id: i32, abil: i32, param: i32, dflt: i32) -> Result<i32, Fault> {
    let mut best_grade = -1;
    let mut value = dflt;

    if has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(value);
    }

    let mut slot = 0i32;

    loop {
        let slot_count = if ctx.orb_store.slot_counts.contains_key(&unit_id) {
            *ctx.orb_store.slot_counts.get(&unit_id).ok_or(Fault::KeyNotFound { site: "get_orb_value_max", key: unit_id as i64 })?
        } else {
            0
        };

        if slot >= slot_count {
            return Ok(value);
        }

        let orb_index = get_equipped_orb(ctx, unit_id, slot)?;

        if orb_index != -1 {
            let orb = ctx.orb_store.orbs.get(orb_index as usize).ok_or(Fault::IndexOutOfRange {
                site: "get_orb_value_max",
                index: orb_index as i64,
                limit: ctx.orb_store.orbs.len() as i64,
            })?;

            if orb.abil == abil && orb.grade > best_grade {
                value = *orb.values.get(param as usize).ok_or(Fault::IndexOutOfRange {
                    site: "get_orb_value_max",
                    index: param as i64,
                    limit: orb.values.len() as i64,
                })?;
                best_grade = orb.grade;
            }
        }

        slot += 1;
    }
}
